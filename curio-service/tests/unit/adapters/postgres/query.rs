use super::*;
use crate::postgres_test_support::PostgresFixture;
use sqlx::{Encode, Postgres, Type};
use std::num::NonZeroU32;

#[derive(Clone, Copy)]
enum Column {
    Id,
    Name,
    Email,
    Avatar,
    UpdatedAt,
}
impl SqlColumn for Column {
    fn sql(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::Email => "email",
            Self::Avatar => "avatar_url",
            Self::UpdatedAt => "updated_at",
        }
    }
}

#[test]
fn optional_predicates_keep_contiguous_binds_and_ordering_is_applied_last() {
    let query = Select::from("users")
        .columns([Column::Id])
        .order_by(Column::Name.desc())
        .where_optional(None::<&str>.map(|value| Column::Email.is_equal_to(value)))
        .where_(Column::Name.is_equal_to("Robert'); DROP TABLE users;--"))
        .where_optional(Some("a").map(|value| Column::Id.is_greater_than(value)))
        .where_(Column::Avatar.is_null())
        .page(NonZeroU32::new(10).unwrap(), 5)
        .build()
        .unwrap();
    assert_eq!(
        query.sql(),
        "SELECT id FROM users WHERE (name = $1) AND (id > $2) AND (avatar_url IS NULL) ORDER BY name DESC LIMIT $3 OFFSET $4"
    );
    assert!(!query.sql().contains("Robert"));
}

#[test]
fn no_filters_has_no_dangling_where_clause() {
    let query = Select::from("users")
        .columns([Column::Id])
        .where_optional(None::<String>.map(|value| Column::Id.is_equal_to(value)))
        .build()
        .unwrap();
    assert_eq!(query.sql(), "SELECT id FROM users");
}

#[test]
fn insert_pairs_columns_and_slots_in_call_order() {
    let query = Insert::into("users")
        .value(Field::Email("email"))
        .value(Field::Avatar(None))
        .value(Field::Id("id"))
        .value(Field::Name("name"))
        .on_conflict(
            Column::Id,
            [
                (Column::Name, Column::Name.of("EXCLUDED")),
                (Column::UpdatedAt, current_timestamp()),
            ],
        )
        .returning([Column::Id, Column::Name])
        .build()
        .unwrap();
    assert_eq!(
        query.sql(),
        "INSERT INTO users (email, avatar_url, id, name) VALUES ($1, $2, $3, $4) ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, updated_at = CURRENT_TIMESTAMP RETURNING id, name"
    );
}

#[test]
fn insert_rejects_empty_and_duplicate_columns() {
    assert!(Insert::into("users").build().is_err());
    assert!(
        Insert::into("users")
            .value(Field::Id("a"))
            .value(Field::Id("b"))
            .build()
            .is_err()
    );
}

struct BadValue;
impl Type<Postgres> for BadValue {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as Type<Postgres>>::type_info()
    }
}
impl<'q> Encode<'q, Postgres> for BadValue {
    fn encode_by_ref(
        &self,
        _: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        Err("test encoding failure".into())
    }
}

#[test]
fn encoding_errors_are_returned_without_panicking_or_losing_the_first_error() {
    assert!(matches!(
        Insert::into("users")
            .value(Field::Bad(BadValue))
            .value(Field::Id("id"))
            .build(),
        Err(sqlx::Error::Encode(_))
    ));
    assert!(matches!(
        Select::from("users")
            .columns([Column::Id])
            .where_(Column::Name.is_equal_to(BadValue))
            .build(),
        Err(sqlx::Error::Encode(_))
    ));
}

#[tokio::test]
async fn binds_round_trip_reordered_values_nulls_arrays_upserts_and_pagination() {
    let Some(postgres) = PostgresFixture::provision("query_composer_round_trip").await else {
        return;
    };
    let pool = postgres.database().pool();
    let name = "Robert'); DROP TABLE users;--";
    let record: (String, String, String, Option<String>) = Insert::into("users")
        .value(Field::Email("one@example.test"))
        .value(Field::Avatar(None))
        .value(Field::Name(name))
        .value(Field::Id("one"))
        .returning([Column::Id, Column::Name, Column::Email, Column::Avatar])
        .build()
        .unwrap()
        .build_query_as()
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(
        record,
        ("one".into(), name.into(), "one@example.test".into(), None)
    );
    for (id, name) in [("two", "Second"), ("one", "Updated")] {
        Insert::into("users")
            .value(Field::Id(id))
            .value(Field::Name(name))
            .value(Field::Email(&format!("{id}@example.test")))
            .on_conflict(
                Column::Id,
                [
                    (Column::Name, Column::Name.of("EXCLUDED")),
                    (Column::UpdatedAt, current_timestamp()),
                ],
            )
            .build()
            .unwrap()
            .build()
            .execute(pool)
            .await
            .unwrap();
    }
    let rows: Vec<(String, String)> = Select::from("users")
        .columns([Column::Id, Column::Name])
        .where_(Column::Id.is_any_of(vec!["one", "two"]))
        .where_optional(None::<&str>.map(|value| Column::Email.is_equal_to(value)))
        .where_(Column::Avatar.is_null())
        .order_by(Column::Id.asc())
        .page(NonZeroU32::new(1).unwrap(), 0)
        .build()
        .unwrap()
        .build_query_as()
        .fetch_all(pool)
        .await
        .unwrap();
    assert_eq!(rows, vec![("one".into(), "Updated".into())]);
    let empty: Vec<(String,)> = Select::from("users")
        .columns([Column::Id])
        .where_(Column::Id.is_any_of(Vec::<String>::new()))
        .build()
        .unwrap()
        .build_query_as()
        .fetch_all(pool)
        .await
        .unwrap();
    assert!(empty.is_empty());
    let skipped = Insert::into("users")
        .value(Field::Id("one"))
        .value(Field::Name("Ignored"))
        .value(Field::Email("different@example.test"))
        .on_conflict(Column::Id, [])
        .build()
        .unwrap()
        .build()
        .execute(pool)
        .await
        .unwrap();
    assert_eq!(skipped.rows_affected(), 0);
    postgres.cleanup().await;
}

#[test]
fn update_assignments_and_predicates_share_one_bind_sequence() {
    let query = Update::table("users")
        .set(Field::Name("new name"))
        .set(Field::Avatar(None))
        .set(Field::UpdatedAt(current_timestamp()))
        .where_(Column::Id.is_equal_to("owner"))
        .where_(Column::Email.is_equal_to("email"))
        .returning([Column::Id, Column::Name])
        .build()
        .unwrap();
    assert_eq!(
        query.sql(),
        "UPDATE users SET name = $1, avatar_url = $2, updated_at = CURRENT_TIMESTAMP WHERE (id = $3) AND (email = $4) RETURNING id, name"
    );
}

#[test]
fn updates_reject_missing_or_duplicate_assignments_and_encoding_failures() {
    assert!(
        Update::table("users")
            .where_(Column::Id.is_equal_to("id"))
            .build()
            .is_err()
    );
    assert!(
        Update::table("users")
            .set(Field::Name("a"))
            .set(Field::Name("b"))
            .where_(Column::Id.is_equal_to("id"))
            .build()
            .is_err()
    );
    assert!(matches!(
        Update::table("users")
            .set(Field::Bad(BadValue))
            .where_(Column::Id.is_equal_to("id"))
            .build(),
        Err(sqlx::Error::Encode(_))
    ));
}

#[tokio::test]
async fn updates_bind_values_to_columns_and_require_all_predicates_to_match() {
    let Some(postgres) = PostgresFixture::provision("query_update_round_trip").await else {
        return;
    };
    let pool = postgres.database().pool();
    Insert::into("users")
        .value(Field::Id("owner"))
        .value(Field::Name("Original"))
        .value(Field::Email("owner@example.test"))
        .build()
        .unwrap()
        .build()
        .execute(pool)
        .await
        .unwrap();
    let row: (String, String, Option<String>) = Update::table("users")
        .set(Field::Avatar(Some("https://example.test/avatar")))
        .set(Field::Name("Updated"))
        .set(Field::UpdatedAt(current_timestamp()))
        .where_(Column::Id.is_equal_to("owner"))
        .where_(Column::Email.is_equal_to("owner@example.test"))
        .returning([Column::Id, Column::Name, Column::Avatar])
        .build()
        .unwrap()
        .build_query_as()
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(
        row,
        (
            "owner".into(),
            "Updated".into(),
            Some("https://example.test/avatar".into())
        )
    );
    let result = Update::table("users")
        .set(Field::Name("Incorrect"))
        .where_(Column::Id.is_equal_to("owner"))
        .where_(Column::Email.is_equal_to("wrong@example.test"))
        .build()
        .unwrap()
        .build()
        .execute(pool)
        .await
        .unwrap();
    assert_eq!(result.rows_affected(), 0);
    let avatar: Option<String> = Update::table("users")
        .set(Field::Avatar(None))
        .where_(Column::Id.is_equal_to("owner"))
        .returning([Column::Avatar])
        .build()
        .unwrap()
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(avatar, None);
    postgres.cleanup().await;
}

#[test]
fn empty_projection_and_postgres_parameter_overflow_return_errors() {
    assert!(
        Select::from("users")
            .columns([] as [Column; 0])
            .build()
            .is_err()
    );
    let mut query = Select::from("users").columns([Column::Id]);
    for _ in 0..65_536 {
        query = query.where_(Column::Id.is_equal_to("id"));
    }
    assert!(matches!(query.build(), Err(sqlx::Error::Protocol(_))));
}

enum Field<'a> {
    Id(&'a str),
    Name(&'a str),
    Email(&'a str),
    Avatar(Option<&'a str>),
    UpdatedAt(Expr<'a>),
    Bad(BadValue),
}
impl<'a> SqlField<'a> for Field<'a> {
    fn into_field(self) -> WriteField<'a> {
        match self {
            Self::Id(value) => Column::Id.value(value),
            Self::Name(value) => Column::Name.value(value),
            Self::Email(value) => Column::Email.value(value),
            Self::Avatar(value) => Column::Avatar.value(value),
            Self::UpdatedAt(value) => Column::UpdatedAt.expression(value),
            Self::Bad(value) => Column::Name.value(value),
        }
    }
}

#[test]
fn dynamic_field_vectors_bind_heterogeneous_values_with_the_same_api() {
    let fields = vec![Field::Id("owner"), Field::Avatar(None), Field::Name("Ada")];
    let insert = Insert::into("users").values(fields).build().unwrap();
    assert_eq!(
        insert.sql(),
        "INSERT INTO users (id, avatar_url, name) VALUES ($1, $2, $3)"
    );
    let update = Update::table("users")
        .sets([Field::Name("Ada"), Field::Avatar(None)])
        .where_(Column::Id.is_equal_to("owner"))
        .build()
        .unwrap();
    assert_eq!(
        update.sql(),
        "UPDATE users SET name = $1, avatar_url = $2 WHERE (id = $3)"
    );
}
