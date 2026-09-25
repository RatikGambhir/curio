//! INSERT column/value pairing and upsert composition.
use sqlx::{Encode, Postgres, QueryBuilder, Type};

use super::{Assignment, BoundSql, Expression, FieldWriter, SqlColumn, SqlField, push_columns};

/// A column and value are added together, so SQLx's bind order cannot drift from
/// the INSERT column order. SQLx encodes values; it does not validate column types.
#[must_use = "an insert must be built and executed"]
pub struct InsertQuery<C> {
    table: &'static str,
    columns: Vec<C>,
    values: BoundSql,
    conflict: Option<(C, Vec<Assignment<C>>)>,
    returning: Vec<C>,
}

impl<C: SqlColumn> InsertQuery<C> {
    pub fn new(table: &'static str) -> Self {
        Self {
            table,
            columns: Vec::new(),
            values: BoundSql::new(""),
            conflict: None,
            returning: Vec::new(),
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

    /// Empty assignments means `DO NOTHING`; otherwise update the listed fields.
    pub fn on_conflict(
        mut self,
        target: C,
        assignments: impl IntoIterator<Item = Assignment<C>>,
    ) -> Self {
        self.conflict = Some((target, assignments.into_iter().collect()));
        self
    }

    pub fn returning(mut self, columns: impl IntoIterator<Item = C>) -> Self {
        self.returning = columns.into_iter().collect();
        self
    }

    /// Rejects empty/duplicate column lists and propagates SQLx encoding errors.
    pub fn build(self) -> Result<QueryBuilder<'static, Postgres>, sqlx::Error> {
        if self.columns.is_empty() {
            return Err(sqlx::Error::Protocol(
                "an insert needs at least one column".to_owned(),
            ));
        }
        for (index, column) in self.columns.iter().enumerate() {
            if self.columns[..index]
                .iter()
                .any(|previous| previous.sql() == column.sql())
            {
                return Err(sqlx::Error::Protocol("duplicate insert column".to_owned()));
            }
        }
        let mut sql = String::from("INSERT INTO ");
        sql.push_str(self.table);
        sql.push_str(" (");
        push_columns(&mut sql, &self.columns);
        sql.push_str(") VALUES (");
        sql.push_str(&self.values.sql);
        sql.push(')');
        if let Some((target, assignments)) = self.conflict {
            sql.push_str(" ON CONFLICT (");
            sql.push_str(target.sql());
            sql.push_str(") DO ");
            if assignments.is_empty() {
                sql.push_str("NOTHING");
            } else {
                sql.push_str("UPDATE SET ");
                for (index, assignment) in assignments.into_iter().enumerate() {
                    if index > 0 {
                        sql.push_str(", ");
                    }
                    match assignment {
                        Assignment::Excluded(column) => {
                            sql.push_str(column.sql());
                            sql.push_str(" = EXCLUDED.");
                            sql.push_str(column.sql());
                        }
                        Assignment::CurrentTimestamp(column) => {
                            sql.push_str(column.sql());
                            sql.push_str(" = CURRENT_TIMESTAMP");
                        }
                    }
                }
            }
        }
        if !self.returning.is_empty() {
            sql.push_str(" RETURNING ");
            push_columns(&mut sql, &self.returning);
        }
        BoundSql { sql, ..self.values }.build()
    }
}

impl<C: SqlColumn> FieldWriter<C> for InsertQuery<C> {
    fn bind<'args, T>(&mut self, column: C, value: T)
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.column(column);
        self.values.push_bind(value);
    }
    fn expression(&mut self, column: C, expression: Expression) {
        self.column(column);
        self.values.push(expression.sql());
    }
}
impl<C: SqlColumn> InsertQuery<C> {
    fn column(&mut self, column: C) {
        if !self.columns.is_empty() {
            self.values.push(", ");
        }
        self.columns.push(column);
    }
}
