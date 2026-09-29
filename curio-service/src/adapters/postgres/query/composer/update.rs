//! Scoped updates, UPDATE FROM and CTE-backed writes.
use super::super::{SqlColumn, SqlField};
use super::clauses::{assignment_list, cte_prefix, returning, where_clause};
use super::expression::invalid;
use super::{Expr, Select, Source};
use sqlx::{Postgres, QueryBuilder};

/// UPDATE refuses to build without assignments and at least one predicate.
#[must_use]
pub struct Update<'a> {
    table: Source<'a>,
    assignments: Vec<(&'static str, Expr<'a>)>,
    source: Option<Source<'a>>,
    filters: Vec<Expr<'a>>,
    returning: Vec<Expr<'a>>,
    ctes: Vec<(&'static str, Expr<'a>)>,
}
impl<'a> Update<'a> {
    pub fn table(table: impl Into<Source<'a>>) -> Self {
        Self {
            table: table.into(),
            assignments: Vec::new(),
            source: None,
            filters: Vec::new(),
            returning: Vec::new(),
            ctes: Vec::new(),
        }
    }
    /// Assign an expression, including a subquery or SQL function.
    pub fn set_expression(self, column: impl SqlColumn, value: Expr<'a>) -> Self {
        self.set(column.expression(value))
    }
    pub fn sets<F: SqlField<'a>>(mut self, fields: impl IntoIterator<Item = F>) -> Self {
        for field in fields {
            self = self.set(field);
        }
        self
    }
    pub fn set(mut self, field: impl SqlField<'a>) -> Self {
        let field = field.into_field();
        let column = field.column;
        let value = field.value;
        self.assignments.push((column, value));
        self
    }
    pub fn from(mut self, source: impl Into<Source<'a>>) -> Self {
        self.source = Some(source.into());
        self
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
    pub fn with(mut self, name: &'static str, query: Select<'a>) -> Self {
        self.ctes.push((name, query.expression()));
        self
    }
    pub fn build(self) -> Result<QueryBuilder<'static, Postgres>, sqlx::Error> {
        let mut expr = cte_prefix(self.ctes, false)
            .push("UPDATE ")
            .append(self.table.0)
            .push(" SET ")
            .append(assignment_list(self.assignments));
        if self.filters.is_empty() {
            expr = expr.append(invalid("an update needs at least one predicate"));
        }
        if let Some(source) = self.source {
            expr = expr.push(" FROM ").append(source.0);
        }
        returning(where_clause(expr, self.filters), self.returning).build()
    }
}
