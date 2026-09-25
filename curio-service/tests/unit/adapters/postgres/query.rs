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
    let query = SelectQuery::new("users", [Column::Id])
        .order_by(Column::Name, Direction::Descending)
        .filter_optional(Column::Email, Comparison::Equal, None::<&str>)
        .filter(
            Column::Name,
            Comparison::Equal,
            "Robert'); DROP TABLE users;--",
        )
        .filter_optional(Column::Id, Comparison::Greater, Some("a"))
        .filter_null(Column::Avatar)
        .page(NonZeroU32::new(10).unwrap(), 5)
        .build()
        .unwrap();
    assert_eq!(
        query.sql(),
        "SELECT id FROM users WHERE name = $1 AND id > $2 AND avatar_url IS NULL ORDER BY name DESC LIMIT $3 OFFSET $4"
    );
    assert!(!query.sql().contains("Robert"));
}

#[test]
fn no_filters_has_no_dangling_where_clause() {
    let query = SelectQuery::new("users", [Column::Id])
        .filter_optional(Column::Id, Comparison::Equal, None::<String>)
        .build()
        .unwrap();
    assert_eq!(query.sql(), "SELECT id FROM users");
}

#[test]
fn insert_pairs_columns_and_slots_in_call_order() {
    let query = InsertQuery::new("users")
        .value(Field::Email("email"))
        .value(Field::Avatar(None))
        .value(Field::Id("id"))
        .value(Field::Name("name"))
        .on_conflict(
            Column::Id,
            [
                Assignment::Excluded(Column::Name),
                Assignment::CurrentTimestamp(Column::UpdatedAt),
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
    assert!(InsertQuery::<Column>::new("users").build().is_err());
    assert!(
        InsertQuery::new("users")
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
        InsertQuery::new("users")
            .value(Field::Bad(BadValue))
            .value(Field::Id("id"))
            .build(),
        Err(sqlx::Error::Encode(_))
    ));
    assert!(matches!(
        SelectQuery::new("users", [Column::Id])
            .filter(Column::Name, Comparison::Equal, BadValue)
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
    let record: (String, String, String, Option<String>) = InsertQuery::new("users")
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
        InsertQuery::new("users")
            .value(Field::Id(id))
            .value(Field::Name(name))
            .value(Field::Email(&format!("{id}@example.test")))
            .on_conflict(
                Column::Id,
                [
                    Assignment::Excluded(Column::Name),
                    Assignment::CurrentTimestamp(Column::UpdatedAt),
                ],
            )
            .build()
            .unwrap()
            .build()
            .execute(pool)
            .await
            .unwrap();
    }
    let rows: Vec<(String, String)> = SelectQuery::new("users", [Column::Id, Column::Name])
        .filter_any(Column::Id, vec!["one", "two"])
        .filter_optional(Column::Email, Comparison::Equal, None::<&str>)
        .filter_null(Column::Avatar)
        .order_by(Column::Id, Direction::Ascending)
        .page(NonZeroU32::new(1).unwrap(), 0)
        .build()
        .unwrap()
        .build_query_as()
        .fetch_all(pool)
        .await
        .unwrap();
    assert_eq!(rows, vec![("one".into(), "Updated".into())]);
    let empty: Vec<(String,)> = SelectQuery::new("users", [Column::Id])
        .filter_any(Column::Id, Vec::<String>::new())
        .build()
        .unwrap()
        .build_query_as()
        .fetch_all(pool)
        .await
        .unwrap();
    assert!(empty.is_empty());
    let skipped = InsertQuery::new("users")
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
    let query = UpdateQuery::new("users")
        .value(Field::Name("new name"))
        .value(Field::Avatar(None))
        .value(Field::UpdatedAt(Expression::CurrentTimestamp))
        .filter(Column::Id, Comparison::Equal, "owner")
        .filter(Column::Email, Comparison::Equal, "email")
        .returning([Column::Id, Column::Name])
        .build()
        .unwrap();
    assert_eq!(
        query.sql(),
        "UPDATE users SET name = $1, avatar_url = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $3 AND email = $4 RETURNING id, name"
    );
}

#[test]
fn updates_reject_missing_or_duplicate_assignments_and_encoding_failures() {
    assert!(
        UpdateQuery::new("users")
            .filter(Column::Id, Comparison::Equal, "id")
            .build()
            .is_err()
    );
    assert!(
        UpdateQuery::new("users")
            .value(Field::Name("a"))
            .value(Field::Name("b"))
            .filter(Column::Id, Comparison::Equal, "id")
            .build()
            .is_err()
    );
    assert!(matches!(
        UpdateQuery::new("users")
            .value(Field::Bad(BadValue))
            .filter(Column::Id, Comparison::Equal, "id")
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
    InsertQuery::new("users")
        .value(Field::Id("owner"))
        .value(Field::Name("Original"))
        .value(Field::Email("owner@example.test"))
        .build()
        .unwrap()
        .build()
        .execute(pool)
        .await
        .unwrap();
    let row: (String, String, Option<String>) = UpdateQuery::new("users")
        .value(Field::Avatar(Some("https://example.test/avatar")))
        .value(Field::Name("Updated"))
        .value(Field::UpdatedAt(Expression::CurrentTimestamp))
        .filter(Column::Id, Comparison::Equal, "owner")
        .filter(Column::Email, Comparison::Equal, "owner@example.test")
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
    let result = UpdateQuery::new("users")
        .value(Field::Name("Incorrect"))
        .filter(Column::Id, Comparison::Equal, "owner")
        .filter(Column::Email, Comparison::Equal, "wrong@example.test")
        .build()
        .unwrap()
        .build()
        .execute(pool)
        .await
        .unwrap();
    assert_eq!(result.rows_affected(), 0);
    let avatar: Option<String> = UpdateQuery::new("users")
        .value(Field::Avatar(None))
        .filter(Column::Id, Comparison::Equal, "owner")
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
    assert!(SelectQuery::<Column>::new("users", []).build().is_err());
    let mut query = SelectQuery::new("users", [Column::Id]);
    for _ in 0..65_536 {
        query = query.filter(Column::Id, Comparison::Equal, "id");
    }
    assert!(matches!(query.build(), Err(sqlx::Error::Protocol(_))));
}

enum Field<'a> {
    Id(&'a str),
    Name(&'a str),
    Email(&'a str),
    Avatar(Option<&'a str>),
    UpdatedAt(Expression),
    Bad(BadValue),
}
impl SqlField for Field<'_> {
    type Column = Column;
    fn write(self, writer: &mut impl FieldWriter<Column>) {
        match self {
            Self::Id(value) => writer.bind(Column::Id, value),
            Self::Name(value) => writer.bind(Column::Name, value),
            Self::Email(value) => writer.bind(Column::Email, value),
            Self::Avatar(value) => writer.bind(Column::Avatar, value),
            Self::UpdatedAt(value) => writer.expression(Column::UpdatedAt, value),
            Self::Bad(value) => writer.bind(Column::Name, value),
        }
    }
}

#[test]
fn dynamic_field_vectors_bind_heterogeneous_values_with_the_same_api() {
    let fields = vec![Field::Id("owner"), Field::Avatar(None), Field::Name("Ada")];
    let insert = InsertQuery::new("users").values(fields).build().unwrap();
    assert_eq!(
        insert.sql(),
        "INSERT INTO users (id, avatar_url, name) VALUES ($1, $2, $3)"
    );
    let update = UpdateQuery::new("users")
        .values([Field::Name("Ada"), Field::Avatar(None)])
        .filter(Column::Id, Comparison::Equal, "owner")
        .build()
        .unwrap();
    assert_eq!(
        update.sql(),
        "UPDATE users SET name = $1, avatar_url = $2 WHERE id = $3"
    );
}
