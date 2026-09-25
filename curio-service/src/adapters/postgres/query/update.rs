//! UPDATE assignments followed by required scope predicates.
use sqlx::{Encode, Postgres, QueryBuilder, Type};

use super::{BoundSql, Comparison, Expression, FieldWriter, SqlColumn, SqlField, push_columns};

/// Collects UPDATE assignments as column/value pairs.
///
/// Call `filter` after the assignments to obtain an executable update. There is
/// deliberately no `build` on this stage, preventing accidental unscoped writes.
#[must_use = "an update needs assignments, a filter, and execution"]
pub struct UpdateQuery<C> {
    query: BoundSql,
    columns: Vec<C>,
}
impl<C: SqlColumn> UpdateQuery<C> {
    pub fn new(table: &'static str) -> Self {
        let mut query = BoundSql::new("UPDATE ");
        query.push(table).push(" SET ");
        Self {
            query,
            columns: Vec::new(),
        }
    }

    /// Adds one typed, data-carrying field using the shared field binding contract.
    pub fn value(mut self, field: impl SqlField<Column = C>) -> Self {
        field.write(&mut self);
        self
    }

    /// Adds a dynamic sequence of fields without manually tracking bind slots.
    pub fn values(mut self, fields: impl IntoIterator<Item = impl SqlField<Column = C>>) -> Self {
        for field in fields {
            field.write(&mut self);
        }
        self
    }

    /// Ends assignment composition and adds the required first WHERE predicate.
    pub fn filter<'args, T>(
        mut self,
        column: C,
        comparison: Comparison,
        value: T,
    ) -> FilteredUpdate<C>
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        if self.columns.is_empty() {
            self.query.error = Some(sqlx::Error::Protocol(
                "an update needs at least one assignment".to_owned(),
            ));
        }
        self.query
            .push(" WHERE ")
            .push(column.sql())
            .push(comparison.sql())
            .push_bind(value);
        FilteredUpdate {
            query: self.query,
            returning: Vec::new(),
        }
    }

    fn assignment(&mut self, column: C) {
        if self
            .columns
            .iter()
            .any(|previous| previous.sql() == column.sql())
            && self.query.error.is_none()
        {
            self.query.error = Some(sqlx::Error::Protocol("duplicate update column".to_owned()));
        }
        if !self.columns.is_empty() {
            self.query.push(", ");
        }
        self.columns.push(column);
        self.query.push(column.sql()).push(" = ");
    }
}

/// UPDATE with at least one bound predicate. Add any additional owner/resource
/// predicates before building; repositories are responsible for access policy.
#[must_use = "an update must be built and executed"]
pub struct FilteredUpdate<C> {
    query: BoundSql,
    returning: Vec<C>,
}
impl<C: SqlColumn> FilteredUpdate<C> {
    pub fn filter<'args, T>(mut self, column: C, comparison: Comparison, value: T) -> Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.query
            .push(" AND ")
            .push(column.sql())
            .push(comparison.sql())
            .push_bind(value);
        self
    }

    pub fn returning(mut self, columns: impl IntoIterator<Item = C>) -> Self {
        self.returning = columns.into_iter().collect();
        self
    }

    pub fn build(mut self) -> Result<QueryBuilder<'static, Postgres>, sqlx::Error> {
        if !self.returning.is_empty() {
            self.query.push(" RETURNING ");
            push_columns(&mut self.query.sql, &self.returning);
        }
        self.query.build()
    }
}

impl<C: SqlColumn> FieldWriter<C> for UpdateQuery<C> {
    fn bind<'args, T>(&mut self, column: C, value: T)
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.assignment(column);
        self.query.push_bind(value);
    }
    fn expression(&mut self, column: C, expression: Expression) {
        self.assignment(column);
        self.query.push(expression.sql());
    }
}
