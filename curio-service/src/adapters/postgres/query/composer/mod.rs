//! Fluent SQL builders grouped by expression and statement responsibilities.

mod clauses;
mod field;
pub use field::WriteField;
mod delete;
mod expression;
mod insert;
mod select;
mod source;
mod transaction;

pub(super) use expression::qualified;
mod update;

pub use delete::Delete;
pub use expression::{
    Expr, Function, SqlType, all, all_of, call, case_when, col, current_timestamp, int,
    millisecond, null, tuple, val,
};
pub use insert::Insert;
pub use select::Select;
pub use source::{Order, Source, rows_from, table};
pub use transaction::repeatable_read;
pub use update::Update;

#[cfg(test)]
#[path = "../../../../../tests/unit/adapters/postgres/composer.rs"]
mod tests;
