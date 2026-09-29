//! Tables, function row sources, aliases and ORDER BY expressions.
use super::super::SqlColumn;
use super::expression::{Expr, identifier, list};

/// One ORDER BY expression and its direction.
pub struct Order<'a>(pub(super) Expr<'a>);
/// A table, set-returning function, or derived query in FROM/JOIN.
pub struct Source<'a>(pub(super) Expr<'a>);
impl<'a> Source<'a> {
    pub fn alias(self, name: &'static str) -> Self {
        Self(self.0.alias_as(name))
    }
    /// Names the columns of a function row source, as in `AS r(id, rank)`.
    pub fn columns<C: SqlColumn>(self, columns: impl IntoIterator<Item = C>) -> Self {
        Self(
            self.0
                .push("(")
                .append(list(
                    columns.into_iter().map(|column| identifier(column.sql())),
                ))
                .push(")"),
        )
    }
    pub fn with_ordinality(self) -> Self {
        Self(self.0.push(" WITH ORDINALITY"))
    }
}
impl<'a> From<&'static str> for Source<'a> {
    fn from(name: &'static str) -> Self {
        table(name)
    }
}
pub fn table<'a>(name: &'static str) -> Source<'a> {
    Source(identifier(name))
}
pub fn rows_from<'a>(expr: Expr<'a>) -> Source<'a> {
    Source(expr)
}
