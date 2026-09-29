//! Scoped deletion and returned records.
use super::clauses::{returning, where_clause};
use super::expression::invalid;
use super::{Expr, Source};
use sqlx::{Postgres, QueryBuilder};

/// DELETE refuses to build without at least one predicate.
#[must_use]
pub struct Delete<'a> {
    table: Source<'a>,
    filters: Vec<Expr<'a>>,
    returning: Vec<Expr<'a>>,
}
impl<'a> Delete<'a> {
    pub fn from(table: impl Into<Source<'a>>) -> Self {
        Self {
            table: table.into(),
            filters: Vec::new(),
            returning: Vec::new(),
        }
    }
    /// Add an AND predicate. Rust reserves `where`, so use `where_`.
    pub fn where_(self, predicate: Expr<'a>) -> Self {
        self.filter(predicate)
    }
    /// Raw-identifier spelling of `where_`.
    pub fn r#where(self, predicate: Expr<'a>) -> Self {
        self.filter(predicate)
    }
    pub fn filter(mut self, predicate: Expr<'a>) -> Self {
        self.filters.push(predicate);
        self
    }
    pub fn returning<E: Into<Expr<'a>>>(mut self, columns: impl IntoIterator<Item = E>) -> Self {
        self.returning = columns.into_iter().map(Into::into).collect();
        self
    }
    pub fn build(self) -> Result<QueryBuilder<'static, Postgres>, sqlx::Error> {
        let mut expr = Expr::sql("DELETE FROM ").append(self.table.0);
        if self.filters.is_empty() {
            expr = expr.append(invalid("a delete needs at least one predicate"));
        }
        returning(where_clause(expr, self.filters), self.returning).build()
    }
}
