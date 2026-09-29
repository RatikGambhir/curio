//! Fluent PostgreSQL composition. Runtime values are always SQLx parameters.
//! Repositories own transactions, row mapping, and authorization predicates.
//!
//! ```
//! use curio_service::adapters::postgres::query::{
//!     Select, Insert, Update, Delete, SqlColumn, sql_columns,
//! };
//! sql_columns! { enum UserColumn { Id => "id", Name => "name", Email => "email" } }
//!
//! let select = Select::from("users")
//!     .columns([UserColumn::Id])
//!     .where_(UserColumn::Name.is_equal_to("Ada"))
//!     .build()?;
//! assert_eq!(select.sql(), "SELECT id FROM users WHERE (name = $1)");
//!
//! let insert = Insert::into("users")
//!     .value(UserColumn::Id.value("user-1"))
//!     .value(UserColumn::Name.value("Ada"))
//!     .value(UserColumn::Email.value("ada@example.test"))
//!     .build()?;
//! let update = Update::table("users")
//!     .set(UserColumn::Email.value("new@example.test"))
//!     .where_(UserColumn::Id.is_equal_to("user-1"))
//!     .returning([UserColumn::Id])
//!     .build()?;
//! let delete = Delete::from("users")
//!     .where_(UserColumn::Id.is_equal_to("user-1"))
//!     .build()?;
//! # Ok::<(), sqlx::Error>(())
//! ```
//!
//! Rust reserves `where`; use `.where_(...)` or `.r#where(...)`. Predicates
//! compose with `.and(...)` / `.or(...)`. For joins and computed values, use
//! `Column::Id.of("alias")`, `col(column)`, `val(value)`, and `column.expression(expr)`.
//! UPDATE and DELETE return a build error without a predicate. This is a guard
//! against accidental unscoped writes, not authorization or a SQL type checker.
//! All statement shapes use the same renderer and fallible SQLx bind allocator.
mod bindings;
mod columns;
mod composer;

use bindings::BoundSql;
pub use columns::sql_columns;
pub use composer::*;

/// Implement with a feature-local enum mapping to trusted SQL identifiers.
pub trait SqlColumn: Copy {
    fn sql(self) -> &'static str;

    /// Qualify a column with a code-owned table name or alias.
    fn of<'a>(self, alias: &'static str) -> Expr<'a> {
        composer::qualified(alias, self.sql())
    }
    fn expression<'a>(self, expression: Expr<'a>) -> WriteField<'a> {
        WriteField::new(self, expression)
    }
    fn value<'a, T>(self, value: T) -> WriteField<'a>
    where
        T: 'a + Send + sqlx::Encode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        self.expression(val(value))
    }
    fn is_null<'a>(self) -> Expr<'a> {
        col(self).is_null()
    }
    fn is_not_null<'a>(self) -> Expr<'a> {
        col(self).is_not_null()
    }
    fn asc<'a>(self) -> Order<'a> {
        col(self).asc()
    }
    fn desc<'a>(self) -> Order<'a> {
        col(self).desc()
    }
    fn is_equal_to<'a, T>(self, value: T) -> Expr<'a>
    where
        T: 'a + Send + sqlx::Encode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        col(self).eq(val(value))
    }
    fn is_not_equal_to<'a, T>(self, value: T) -> Expr<'a>
    where
        T: 'a + Send + sqlx::Encode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        col(self).ne(val(value))
    }
    fn is_less_than<'a, T>(self, value: T) -> Expr<'a>
    where
        T: 'a + Send + sqlx::Encode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        col(self).lt(val(value))
    }
    fn is_less_than_or_equal_to<'a, T>(self, value: T) -> Expr<'a>
    where
        T: 'a + Send + sqlx::Encode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        col(self).le(val(value))
    }
    fn is_greater_than<'a, T>(self, value: T) -> Expr<'a>
    where
        T: 'a + Send + sqlx::Encode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        col(self).gt(val(value))
    }
    fn is_greater_than_or_equal_to<'a, T>(self, value: T) -> Expr<'a>
    where
        T: 'a + Send + sqlx::Encode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        col(self).ge(val(value))
    }
    fn is_any_of<'a, T>(self, value: T) -> Expr<'a>
    where
        T: 'a + Send + sqlx::Encode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        col(self).eq_any(val(value))
    }
}

/// A typed field enum can associate each column with a specific Rust payload.
pub trait SqlField<'a> {
    fn into_field(self) -> WriteField<'a>;
}

/// Allowed ORDER BY directions.
#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Ascending,
    Descending,
}

#[cfg(test)]
#[path = "../../../../tests/unit/adapters/postgres/query.rs"]
mod tests;
