use super::*;
use crate::adapters::postgres::query::{SqlColumn, sql_columns};
sql_columns! { enum Column { Id => "id",OwnerId => "owner_id",Name => "name",Email => "email",Avatar => "avatar_url",Title => "title",ParentId => "parent_id",Rank => "rank",SortOrder => "sort_order",TaskId => "task_id",TagId => "tag_id" } }
use sqlx::{Encode, Postgres, Type};

#[test]
fn nested_queries_allocate_binds_in_sql_order() {
    let query = Select::from(table("tasks").alias("t"))
        .columns([Column::Id.of("t")])
        .where_(
            Column::OwnerId
                .of("t")
                .eq(val("Robert'); DROP TABLE users;--")),
        )
        .where_(
            Select::from("task_tags")
                .columns([int(1)])
                .where_(col(Column::Name).eq(val("tag")))
                .exists(),
        )
        .limit(val(10_i64))
        .build()
        .unwrap();
    assert_eq!(
        query.sql(),
        "SELECT t.id FROM tasks AS t WHERE (t.owner_id = $1) AND EXISTS (SELECT 1 FROM task_tags WHERE (name = $2)) LIMIT $3"
    );
    assert!(!query.sql().contains("Robert"));
}

#[test]
fn writes_require_scope_and_valid_columns() {
    assert!(
        Update::table("tasks")
            .set(Column::Title.value("x"))
            .build()
            .is_err()
    );
    assert!(Delete::from("tasks").build().is_err());
    assert!(
        Insert::into("tasks")
            .value(Column::Id.value("a"))
            .value(Column::Id.value("b"))
            .build()
            .is_err()
    );
    assert!(
        Select::from("tasks")
            .columns([col(InvalidColumn)])
            .build()
            .is_err()
    );
    assert!(
        Insert::rows("tasks", [Column::Id, Column::Title], [vec![val("id")]])
            .build()
            .is_err()
    );
}

#[test]
fn ctes_window_functions_and_insert_select_share_one_bind_sequence() {
    let ranks = Select::from("tasks")
        .columns([
            col(Column::Id),
            call(Function::RowNumber, [])
                .over([col(Column::ParentId)], [col(Column::Id).asc()])
                .alias(Column::Rank),
        ])
        .where_(col(Column::OwnerId).eq(val("owner")));
    let query = Update::table(table("tasks").alias("t"))
        .with("ranks", ranks)
        .set(Column::SortOrder.expression(Column::Rank.of("r").minus(int(1))))
        .set(Column::Title.value("title"))
        .from(table("ranks").alias("r"))
        .where_(Column::Id.of("t").eq(Column::Id.of("r")))
        .where_(Column::OwnerId.of("t").eq(val("owner")))
        .build()
        .unwrap();
    assert_eq!(
        query.sql(),
        "WITH ranks AS (SELECT id, row_number() OVER (PARTITION BY parent_id ORDER BY id ASC) AS rank FROM tasks WHERE (owner_id = $1)) UPDATE tasks AS t SET sort_order = (r.rank - 1), title = $2 FROM ranks AS r WHERE (t.id = r.id) AND (t.owner_id = $3)"
    );
    let query = Insert::from_select(
        "task_tag_assignments",
        [Column::OwnerId, Column::TaskId, Column::TagId],
        Select::expressions([
            val("owner"),
            val("task"),
            call(
                Function::Unnest,
                [val(Vec::<String>::new()).cast(SqlType::TextArray)],
            ),
        ]),
    )
    .build()
    .unwrap();
    assert_eq!(
        query.sql(),
        "INSERT INTO task_tag_assignments (owner_id, task_id, tag_id) SELECT $1, $2, unnest(($3)::text[])"
    );
}

#[test]
fn union_ordering_follows_all_branches_and_optional_window_clauses_are_valid() {
    let query = Select::expressions([val("first").alias(Column::Name)])
        .order_by(col(Column::Name).asc())
        .limit(val(2_i64))
        .union_all(Select::expressions([val("second")]).limit(val(1_i64)))
        .build()
        .unwrap();
    assert_eq!(
        query.sql(),
        "SELECT $1 AS name UNION ALL (SELECT $2 LIMIT $3) ORDER BY name ASC LIMIT $4"
    );
    assert_eq!(
        Select::from("users")
            .columns([call(Function::RowNumber, []).over([], [col(Column::Id).asc()])])
            .build()
            .unwrap()
            .sql(),
        "SELECT row_number() OVER (ORDER BY id ASC) FROM users"
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
fn select_starts_from_its_source_and_appends_columns_in_order() {
    let query = Select::from(table("tasks").alias("t"))
        .columns([Column::Id.of("t")])
        .columns([Column::Title.of("t")])
        .where_(Column::OwnerId.of("t").eq(val("owner")))
        .build()
        .unwrap();
    assert_eq!(
        query.sql(),
        "SELECT t.id, t.title FROM tasks AS t WHERE (t.owner_id = $1)"
    );
    assert!(Select::from("tasks").build().is_err());
}

#[test]
fn invalid_shapes_encoding_failures_and_parameter_overflow_return_errors() {
    assert!(Select::expressions([] as [Expr; 0]).build().is_err());
    assert!(
        Update::table("tasks")
            .where_(col(Column::Id).eq(val("id")))
            .build()
            .is_err()
    );
    assert!(
        Update::table("tasks")
            .set(Column::Title.value("a"))
            .set(Column::Title.value("b"))
            .where_(col(Column::Id).eq(val("id")))
            .build()
            .is_err()
    );
    assert!(
        Insert::rows("tasks", [Column::Id], [])
            .value(Column::Title.value("title"))
            .build()
            .is_err()
    );
    assert!(matches!(
        Select::expressions([val(BadValue)]).build(),
        Err(sqlx::Error::Encode(_))
    ));
    let query = Insert::rows("users", [Column::Id], (0..65_536).map(|_| vec![val("id")]));
    assert!(matches!(query.build(), Err(sqlx::Error::Protocol(_))));
}

#[tokio::test]
async fn grouped_predicates_batch_upserts_and_mutations_round_trip() {
    use crate::postgres_test_support::PostgresFixture;
    let Some(postgres) = PostgresFixture::provision("fluent_query_round_trip").await else {
        return;
    };
    let pool = postgres.database().pool();
    let untrusted = "Robert'); DROP TABLE users;--";
    Insert::rows(
        "users",
        [Column::Id, Column::Name, Column::Email, Column::Avatar],
        [
            vec![
                val("a"),
                val(untrusted),
                val("a@example.test"),
                val(None::<&str>),
            ],
            vec![
                val("b"),
                val("Other"),
                val("b@example.test"),
                val(Some("avatar")),
            ],
        ],
    )
    .build()
    .unwrap()
    .build()
    .execute(pool)
    .await
    .unwrap();
    let names: Vec<String> = Select::from("users")
        .columns([col(Column::Name)])
        .where_(col(Column::Id).eq_any(val(vec!["a", "b"])))
        .where_(
            col(Column::Name)
                .eq(val(untrusted))
                .or(col(Column::Name).eq(val("Missing"))),
        )
        .where_(col(Column::Avatar).is_null())
        .build()
        .unwrap()
        .build_query_scalar()
        .fetch_all(pool)
        .await
        .unwrap();
    assert_eq!(names, [untrusted]);
    let updated: String = Insert::into("users")
        .value(Column::Id.value("a"))
        .value(Column::Name.value("Changed"))
        .value(Column::Email.value("a@example.test"))
        .on_conflict(Column::Id, [(Column::Name, Column::Name.of("EXCLUDED"))])
        .returning([col(Column::Name)])
        .build()
        .unwrap()
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(updated, "Changed");
    let no_update = Update::table("users")
        .set(Column::Name.value("Forbidden"))
        .where_(col(Column::Id).eq(val("a")))
        .where_(col(Column::Email).eq(val("b@example.test")))
        .build()
        .unwrap()
        .build()
        .execute(pool)
        .await
        .unwrap();
    assert_eq!(no_update.rows_affected(), 0);
    let empty: Vec<String> = Select::from("users")
        .columns([col(Column::Id)])
        .where_(col(Column::Id).eq_any(val(Vec::<String>::new())))
        .build()
        .unwrap()
        .build_query_scalar()
        .fetch_all(pool)
        .await
        .unwrap();
    assert!(empty.is_empty());
    let deleted: Vec<String> = Delete::from("users")
        .where_(col(Column::Id).eq(val("a")))
        .returning([col(Column::Id)])
        .build()
        .unwrap()
        .build_query_scalar()
        .fetch_all(pool)
        .await
        .unwrap();
    assert_eq!(deleted, ["a"]);
    let remaining: Vec<String> = Select::from("users")
        .columns([col(Column::Id)])
        .build()
        .unwrap()
        .build_query_scalar()
        .fetch_all(pool)
        .await
        .unwrap();
    assert_eq!(remaining, ["b"]);
    postgres.cleanup().await;
}

#[derive(Clone, Copy)]
struct InvalidColumn;
impl SqlColumn for InvalidColumn {
    fn sql(self) -> &'static str {
        "id; DROP TABLE tasks"
    }
}

#[tokio::test]
async fn cte_left_join_grouping_having_and_pagination_round_trip() {
    use crate::postgres_test_support::PostgresFixture;
    let Some(postgres) = PostgresFixture::provision("grouped_query_round_trip").await else {
        return;
    };
    let pool = postgres.database().pool();
    Insert::rows(
        "users",
        [Column::Id, Column::Name, Column::Email],
        [
            vec![val("a"), val("same"), val("a@example.test")],
            vec![val("b"), val("same"), val("b@example.test")],
            vec![val("c"), val("other"), val("c@example.test")],
        ],
    )
    .build()
    .unwrap()
    .build()
    .execute(pool)
    .await
    .unwrap();
    let matches = || call(Function::Count, [Column::Id.of("peer")]);
    let mut query = Select::from(table("selected").alias("u"))
        .with(
            "selected",
            Select::from("users")
                .columns([Column::Id, Column::Name])
                .where_(Column::Name.is_equal_to("same")),
        )
        .columns([Column::Id.of("u"), matches().alias(Column::Rank)])
        .distinct()
        .left_join(
            table("users").alias("peer"),
            Column::Name.of("u").eq(Column::Name.of("peer")),
        )
        .r#where(Column::Id.of("u").ne(val("excluded")))
        .where_optional(None)
        .group_by([Column::Id.of("u")])
        .having(matches().gt(val(1_i64)))
        .order_by(Column::Id.of("u").asc())
        .page(std::num::NonZeroU32::new(1).unwrap(), 1)
        .build()
        .unwrap();
    assert_eq!(
        query.sql(),
        "WITH selected AS (SELECT id, name FROM users WHERE (name = $1)) SELECT DISTINCT u.id, count(peer.id) AS rank FROM selected AS u LEFT JOIN users AS peer ON (u.name = peer.name) WHERE (u.id <> $2) GROUP BY u.id HAVING (count(peer.id) > $3) ORDER BY u.id ASC LIMIT $4 OFFSET $5"
    );
    let rows: Vec<(String, i64)> = query.build_query_as().fetch_all(pool).await.unwrap();
    assert_eq!(rows, [("b".to_owned(), 2)]);
    postgres.cleanup().await;
}
