//! Structured expressions shared by the statement builders.

mod functions;
mod operators;
mod render;
mod types;
mod values;
mod window;

pub use functions::{Function, call, case_when};
use render::Part;
pub(super) use render::{invalid, list};
pub use types::SqlType;
pub(super) use values::identifier;
pub(in super::super) use values::qualified;
pub use values::{all, all_of, col, current_timestamp, int, millisecond, null, tuple, val};

/// A structured SQL expression. There is deliberately no raw SQL constructor.
#[must_use]
pub struct Expr<'a> {
    parts: Vec<Part<'a>>,
}
impl<'a> Expr<'a> {
    pub(super) fn sql(sql: &'static str) -> Self {
        Self {
            parts: vec![Part::Sql(sql)],
        }
    }
    pub(super) fn append(mut self, other: Self) -> Self {
        self.parts.extend(other.parts);
        self
    }
    pub(super) fn push(self, sql: &'static str) -> Self {
        self.append(Self::sql(sql))
    }
    pub(super) fn grouped(self) -> Self {
        Self::sql("(").append(self).push(")")
    }
}
