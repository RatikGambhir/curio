//! Enum-driven PostgreSQL SELECT, INSERT and UPDATE composition for feature repositories.
//!
//! Columns and expressions are code-owned enums; table names are static SQL.
//! Runtime values always use SQLx binds. This is deliberately not an ORM:
//! repositories retain row mapping, transactions, and domain-specific SQL.
//!
//! # Examples
//!
//! Data-carrying field enums associate each column with its Rust value type.
//! The same `.value(...)` API works for INSERT and UPDATE. SQLx encodes values
//! and generates bind slots; no caller counts placeholders.
//!
//! ```
//! use curio_service::query::{Comparison, FieldWriter, InsertQuery, SqlColumn, SqlField, UpdateQuery};
//!
//! #[derive(Clone, Copy)]
//! enum UserColumn { Id, Email }
//! impl SqlColumn for UserColumn {
//!     fn sql(self) -> &'static str {
//!         match self { Self::Id => "id", Self::Email => "email" }
//!     }
//! }
//! enum UserField<'a> { Id(&'a str), Email(&'a str) }
//! impl SqlField for UserField<'_> {
//!     type Column = UserColumn;
//!     fn write(self, writer: &mut impl FieldWriter<UserColumn>) {
//!         match self {
//!             Self::Id(value) => writer.bind(UserColumn::Id, value),
//!             Self::Email(value) => writer.bind(UserColumn::Email, value),
//!         }
//!     }
//! }
//!
//! let insert = InsertQuery::new("users")
//!     .value(UserField::Email("ada@example.test"))
//!     .value(UserField::Id("user-1"))
//!     .build()?;
//! assert_eq!(insert.sql(), "INSERT INTO users (email, id) VALUES ($1, $2)");
//!
//! let update = UpdateQuery::new("users")
//!     .value(UserField::Email("grace@example.test"))
//!     .filter(UserColumn::Id, Comparison::Equal, "user-1")
//!     .returning([UserColumn::Id, UserColumn::Email])
//!     .build()?;
//! assert_eq!(update.sql(), "UPDATE users SET email = $1 WHERE id = $2 RETURNING id, email");
//! # Ok::<(), sqlx::Error>(())
//! ```
//!
//! An unfiltered UPDATE cannot be built:
//!
//! ```compile_fail
//! use curio_service::query::{FieldWriter, SqlColumn, SqlField, UpdateQuery};
//! #[derive(Clone, Copy)]
//! enum UserColumn { Email }
//! impl SqlColumn for UserColumn {
//!     fn sql(self) -> &'static str { "email" }
//! }
//! enum UserField<'a> { Email(&'a str) }
//! impl SqlField for UserField<'_> {
//!     type Column = UserColumn;
//!     fn write(self, writer: &mut impl FieldWriter<UserColumn>) {
//!         let Self::Email(value) = self;
//!         writer.bind(UserColumn::Email, value);
//!     }
//! }
//! UpdateQuery::new("users").value(UserField::Email("ada@example.test")).build();
//! ```
mod bindings;
mod insert;
mod select;
mod update;

use bindings::BoundSql;
pub use insert::InsertQuery;
pub use select::SelectQuery;
pub use update::{FilteredUpdate, UpdateQuery};

/// Implement with a feature-local enum. Each variant maps to trusted SQL.
pub trait SqlColumn: Copy {
    /// Trusted, correctly quoted SQL identifier for this column.
    fn sql(self) -> &'static str;
}

/// A feature-local data-carrying enum that maps each variant to one typed field.
/// Implement `write` with an exhaustive match, binding the variant's payload.
pub trait SqlField {
    type Column: SqlColumn;
    fn write(self, writer: &mut impl FieldWriter<Self::Column>);
}

/// Binding target used by `SqlField` implementations. INSERT and UPDATE share
/// this contract, keeping field-to-column mappings in one place.
pub trait FieldWriter<C: SqlColumn> {
    /// Bind a field payload using SQLx's encoder and next placeholder.
    fn bind<'args, T>(&mut self, column: C, value: T)
    where
        T: 'args + sqlx::Encode<'args, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>;

    /// Write an allowlisted SQL expression instead of binding a runtime value.
    fn expression(&mut self, column: C, expression: Expression);
}

/// Allowlisted comparison operators; SQL operators never come from request text.
#[derive(Clone, Copy, Debug)]
pub enum Comparison {
    Equal,
    Less,
    Greater,
}

impl Comparison {
    fn sql(self) -> &'static str {
        match self {
            Self::Equal => " = ",
            Self::Less => " < ",
            Self::Greater => " > ",
        }
    }
}

/// Allowed ORDER BY directions.
#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Ascending,
    Descending,
}

/// An upsert assignment with no interpolated runtime SQL.
#[derive(Clone, Copy, Debug)]
pub enum Assignment<C> {
    /// Copy this column from the attempted insert row.
    Excluded(C),
    /// Set this column to the transaction timestamp.
    CurrentTimestamp(C),
}

/// Trusted expressions supported by writes. Runtime text is always a bind value.
#[derive(Clone, Copy, Debug)]
pub enum Expression {
    CurrentTimestamp,
}
impl Expression {
    fn sql(self) -> &'static str {
        match self {
            Self::CurrentTimestamp => "CURRENT_TIMESTAMP",
        }
    }
}

fn push_columns<C: SqlColumn>(sql: &mut String, columns: &[C]) {
    for (index, column) in columns.iter().enumerate() {
        if index > 0 {
            sql.push_str(", ");
        }
        sql.push_str(column.sql());
    }
}

#[cfg(test)]
#[path = "../../tests/unit/query.rs"]
mod tests;
