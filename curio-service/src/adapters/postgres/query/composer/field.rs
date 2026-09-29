//! One write assignment shared by INSERT and UPDATE.
use super::super::{SqlColumn, SqlField};
use super::Expr;

#[must_use]
pub struct WriteField<'a> {
    pub(super) column: &'static str,
    pub(super) value: Expr<'a>,
}
impl<'a> WriteField<'a> {
    pub fn new(column: impl SqlColumn, value: Expr<'a>) -> Self {
        Self {
            column: column.sql(),
            value,
        }
    }
}
impl<'a> SqlField<'a> for WriteField<'a> {
    fn into_field(self) -> Self {
        self
    }
}
