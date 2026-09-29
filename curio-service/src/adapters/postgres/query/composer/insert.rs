//! Single-row, batch, SELECT and conflict-handled inserts.
use super::super::{SqlColumn, SqlField};
use super::clauses::{assignment_list, returning, validate_columns};
use super::expression::{identifier, invalid, list};
use super::{Expr, Select};
use sqlx::{Postgres, QueryBuilder};

/// INSERT supports column/value pairs, multiple rows, SELECT and upserts.
#[must_use]
pub struct Insert<'a> {
    table: &'static str,
    columns: Vec<&'static str>,
    rows: Vec<Vec<Expr<'a>>>,
    source: Option<Select<'a>>,
    conflict: Option<(&'static str, Vec<(&'static str, Expr<'a>)>)>,
    returning: Vec<Expr<'a>>,
}
impl<'a> Insert<'a> {
    pub fn into(table: &'static str) -> Self {
        Self {
            table,
            columns: Vec::new(),
            rows: vec![Vec::new()],
            source: None,
            conflict: None,
            returning: Vec::new(),
        }
    }
    /// Assign an expression, including a subquery or SQL function.
    pub fn value_expression(self, column: impl SqlColumn, value: Expr<'a>) -> Self {
        self.value(column.expression(value))
    }
    pub fn values<F: SqlField<'a>>(mut self, fields: impl IntoIterator<Item = F>) -> Self {
        for field in fields {
            self = self.value(field);
        }
        self
    }
    pub fn value(mut self, field: impl SqlField<'a>) -> Self {
        let field = field.into_field();
        let column = field.column;
        let value = field.value;
        self.columns.push(column);
        if self.source.is_some() || self.rows.len() != 1 {
            self.source = None;
            self.rows = vec![vec![invalid(
                "value cannot be added to a batch or insert-select",
            )]];
        } else {
            self.rows[0].push(value);
        }
        self
    }
    pub fn rows<C: SqlColumn>(
        table: &'static str,
        columns: impl IntoIterator<Item = C>,
        rows: impl IntoIterator<Item = Vec<Expr<'a>>>,
    ) -> Self {
        Self {
            table,
            columns: columns.into_iter().map(SqlColumn::sql).collect(),
            rows: rows.into_iter().collect(),
            source: None,
            conflict: None,
            returning: Vec::new(),
        }
    }
    pub fn from_select<C: SqlColumn>(
        table: &'static str,
        columns: impl IntoIterator<Item = C>,
        query: Select<'a>,
    ) -> Self {
        Self {
            table,
            columns: columns.into_iter().map(SqlColumn::sql).collect(),
            rows: Vec::new(),
            source: Some(query),
            conflict: None,
            returning: Vec::new(),
        }
    }
    /// Use `column.of("EXCLUDED")` to copy the attempted insert value.
    pub fn on_conflict<C: SqlColumn>(
        mut self,
        target: C,
        assignments: impl IntoIterator<Item = (C, Expr<'a>)>,
    ) -> Self {
        let assignments = assignments
            .into_iter()
            .map(|(column, value)| (column.sql(), value))
            .collect();
        self.conflict = Some((target.sql(), assignments));
        self
    }
    pub fn returning<E: Into<Expr<'a>>>(mut self, columns: impl IntoIterator<Item = E>) -> Self {
        self.returning = columns.into_iter().map(Into::into).collect();
        self
    }
    pub fn build(self) -> Result<QueryBuilder<'static, Postgres>, sqlx::Error> {
        let mut expr = Expr::sql("INSERT INTO ")
            .append(identifier(self.table))
            .push(" (")
            .append(list(self.columns.iter().copied().map(identifier)))
            .push(") ");
        expr = validate_columns(expr, &self.columns);
        if let Some(source) = self.source {
            expr = expr.append(source.expression());
        } else {
            if self.rows.is_empty() || self.rows.iter().any(|row| row.len() != self.columns.len()) {
                expr = expr.append(invalid("insert row width does not match columns"));
            }
            expr = expr
                .push("VALUES ")
                .append(list(self.rows.into_iter().map(|row| list(row).grouped())));
        }
        if let Some((target, assignments)) = self.conflict {
            expr = expr
                .push(" ON CONFLICT (")
                .append(identifier(target))
                .push(") DO ");
            expr = if assignments.is_empty() {
                expr.push("NOTHING")
            } else {
                expr.push("UPDATE SET ")
                    .append(assignment_list(assignments))
            };
        }
        returning(expr, self.returning).build()
    }
}
