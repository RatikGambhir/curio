//! SELECT projections, optional filters, ordering and pagination.
use std::num::NonZeroU32;

use sqlx::{Encode, Postgres, QueryBuilder, Type};

use super::{BoundSql, Comparison, Direction, SqlColumn, push_columns};

/// Composes optional AND predicates, ordering and bound pagination.
///
/// An absent optional filter means no predicate, not `IS NULL`. Use
/// `filter_null` to select SQL NULL explicitly. Ordering is appended at build
/// time, so filters can be added independently of sort/pagination decisions.
#[must_use = "a query must be built and executed"]
pub struct SelectQuery<C> {
    query: BoundSql,
    has_predicate: bool,
    ordering: Vec<(C, Direction)>,
    page: Option<(NonZeroU32, u32)>,
}

impl<C: SqlColumn> SelectQuery<C> {
    /// Selects enum columns from a code-owned table name.
    pub fn new(table: &'static str, columns: impl IntoIterator<Item = C>) -> Self {
        let columns: Vec<_> = columns.into_iter().collect();
        let mut query = BoundSql::new("SELECT ");
        if columns.is_empty() {
            query.error = Some(sqlx::Error::Protocol(
                "a select needs at least one column".to_owned(),
            ));
        }
        push_columns(&mut query.sql, &columns);
        query.push(" FROM ").push(table);
        Self {
            query,
            has_predicate: false,
            ordering: Vec::new(),
            page: None,
        }
    }

    pub fn filter<'args, T>(mut self, column: C, comparison: Comparison, value: T) -> Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.predicate();
        self.query
            .push(column.sql())
            .push(comparison.sql())
            .push_bind(value);
        self
    }

    pub fn filter_optional<'args, T>(
        self,
        column: C,
        comparison: Comparison,
        value: Option<T>,
    ) -> Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        match value {
            Some(value) => self.filter(column, comparison, value),
            None => self,
        }
    }

    pub fn filter_null(mut self, column: C) -> Self {
        self.predicate();
        self.query.push(column.sql()).push(" IS NULL");
        self
    }

    /// Uses PostgreSQL ANY with one array bind, including an empty array (false).
    pub fn filter_any<'args, T>(mut self, column: C, values: Vec<T>) -> Self
    where
        Vec<T>: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.predicate();
        self.query
            .push(column.sql())
            .push(" = ANY(")
            .push_bind(values)
            .push(")");
        self
    }

    pub fn order_by(mut self, column: C, direction: Direction) -> Self {
        self.ordering.push((column, direction));
        self
    }

    /// Replaces pagination; callers choose their own domain-specific maximum.
    pub fn page(mut self, limit: NonZeroU32, offset: u32) -> Self {
        self.page = Some((limit, offset));
        self
    }

    pub fn build(mut self) -> Result<QueryBuilder<'static, Postgres>, sqlx::Error> {
        if !self.ordering.is_empty() {
            self.query.push(" ORDER BY ");
            for (index, (column, direction)) in self.ordering.into_iter().enumerate() {
                if index > 0 {
                    self.query.push(", ");
                }
                self.query.push(column.sql()).push(match direction {
                    Direction::Ascending => " ASC",
                    Direction::Descending => " DESC",
                });
            }
        }
        if let Some((limit, offset)) = self.page {
            self.query.push(" LIMIT ").push_bind(i64::from(limit.get()));
            self.query.push(" OFFSET ").push_bind(i64::from(offset));
        }
        self.query.build()
    }

    fn predicate(&mut self) {
        self.query.push(if self.has_predicate {
            " AND "
        } else {
            " WHERE "
        });
        self.has_predicate = true;
    }
}
