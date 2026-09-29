//! Transaction setup statements used by feature repositories.
use sqlx::{Postgres, QueryBuilder};

/// Start a consistent read-only snapshot using the same query layer.
pub fn repeatable_read() -> QueryBuilder<'static, Postgres> {
    QueryBuilder::new("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
}
