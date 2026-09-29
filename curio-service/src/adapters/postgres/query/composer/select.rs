//! Read queries, joins, pagination, locks and recursive unions.
use super::clauses::{cte_prefix, where_clause};
use super::expression::{invalid, list};
use super::{Expr, Order, Source, val};
use sqlx::{Postgres, QueryBuilder};
use std::num::NonZeroU32;

/// SELECT with grouped filters, joins, CTEs, keyset ordering and row locks.
#[must_use]
pub struct Select<'a> {
    projection: Vec<Expr<'a>>,
    source: Option<Source<'a>>,
    joins: Vec<Expr<'a>>,
    filters: Vec<Expr<'a>>,
    order: Vec<Order<'a>>,
    limit: Option<Expr<'a>>,
    offset: Option<Expr<'a>>,
    group: Vec<Expr<'a>>,
    having: Vec<Expr<'a>>,
    distinct: bool,
    lock: bool,
    unions: Vec<Select<'a>>,
    ctes: Vec<(&'static str, Expr<'a>)>,
    recursive: bool,
}
impl<'a> Select<'a> {
    /// Starts a query over a table, row source or aliased source; add the
    /// projection with `columns`.
    pub fn from(source: impl Into<Source<'a>>) -> Self {
        Self::new(Some(source.into()), Vec::new())
    }
    /// A FROM-less SELECT of expressions, such as `SELECT EXISTS (...)` or the
    /// bound row of an insert-select.
    pub fn expressions<E: Into<Expr<'a>>>(projection: impl IntoIterator<Item = E>) -> Self {
        Self::new(None, projection.into_iter().map(Into::into).collect())
    }
    fn new(source: Option<Source<'a>>, projection: Vec<Expr<'a>>) -> Self {
        Self {
            projection,
            source,
            joins: Vec::new(),
            filters: Vec::new(),
            order: Vec::new(),
            limit: None,
            offset: None,
            group: Vec::new(),
            having: Vec::new(),
            distinct: false,
            lock: false,
            unions: Vec::new(),
            ctes: Vec::new(),
            recursive: false,
        }
    }
    /// Appends projected column enums or expressions in order.
    pub fn columns<E: Into<Expr<'a>>>(mut self, projection: impl IntoIterator<Item = E>) -> Self {
        self.projection
            .extend(projection.into_iter().map(Into::into));
        self
    }
    pub fn join(mut self, source: impl Into<Source<'a>>, on: Expr<'a>) -> Self {
        self.joins.push(
            Expr::sql(" JOIN ")
                .append(source.into().0)
                .push(" ON ")
                .append(on),
        );
        self
    }
    pub fn cross_join(mut self, source: impl Into<Source<'a>>) -> Self {
        self.joins
            .push(Expr::sql(" CROSS JOIN ").append(source.into().0));
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
    pub fn order_by(mut self, order: Order<'a>) -> Self {
        self.order.push(order);
        self
    }
    pub fn limit(mut self, limit: impl Into<Expr<'a>>) -> Self {
        self.limit = Some(limit.into());
        self
    }
    pub fn where_optional(self, predicate: Option<Expr<'a>>) -> Self {
        match predicate {
            Some(predicate) => self.where_(predicate),
            None => self,
        }
    }
    pub fn offset(mut self, offset: Expr<'a>) -> Self {
        self.offset = Some(offset);
        self
    }
    pub fn page(self, limit: NonZeroU32, offset: u32) -> Self {
        self.limit(val(i64::from(limit.get())))
            .offset(val(i64::from(offset)))
    }
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }
    pub fn group_by<E: Into<Expr<'a>>>(mut self, columns: impl IntoIterator<Item = E>) -> Self {
        self.group.extend(columns.into_iter().map(Into::into));
        self
    }
    pub fn having(mut self, predicate: Expr<'a>) -> Self {
        self.having.push(predicate);
        self
    }
    pub fn with(mut self, name: &'static str, query: Self) -> Self {
        self.ctes.push((name, query.expression()));
        self
    }
    pub fn left_join(mut self, source: impl Into<Source<'a>>, on: Expr<'a>) -> Self {
        self.joins.push(
            Expr::sql(" LEFT JOIN ")
                .append(source.into().0)
                .push(" ON ")
                .append(on),
        );
        self
    }
    pub fn for_update(mut self) -> Self {
        self.lock = true;
        self
    }
    pub fn union_all(mut self, query: Self) -> Self {
        self.unions.push(query);
        self
    }
    pub fn with_recursive(mut self, name: &'static str, query: Self) -> Self {
        self.recursive = true;
        self.ctes.push((name, query.expression()));
        self
    }
    pub fn scalar(self) -> Expr<'a> {
        self.expression().grouped()
    }
    pub fn exists(self) -> Expr<'a> {
        Expr::sql("EXISTS ").append(self.scalar())
    }
    pub fn lateral(self, alias: &'static str) -> Source<'a> {
        Source(Expr::sql("LATERAL ").append(self.scalar()).alias_as(alias))
    }
    pub fn build(self) -> Result<QueryBuilder<'static, Postgres>, sqlx::Error> {
        self.expression().build()
    }
    pub(super) fn expression(self) -> Expr<'a> {
        let mut expr = cte_prefix(self.ctes, self.recursive).push("SELECT ");
        if self.distinct {
            expr = expr.push("DISTINCT ");
        }
        if self.projection.is_empty() {
            expr = expr.append(invalid("a select needs at least one projection"));
        }
        expr = expr.append(list(self.projection));
        if let Some(source) = self.source {
            expr = expr.push(" FROM ").append(source.0);
        }
        for join in self.joins {
            expr = expr.append(join);
        }
        expr = where_clause(expr, self.filters);
        if !self.group.is_empty() {
            expr = expr.push(" GROUP BY ").append(list(self.group));
        }
        for (index, predicate) in self.having.into_iter().enumerate() {
            expr = expr
                .push(if index == 0 { " HAVING " } else { " AND " })
                .append(predicate);
        }
        for union in self.unions {
            expr = expr
                .push(" UNION ALL ")
                .append(union.expression().grouped());
        }
        if !self.order.is_empty() {
            expr = expr
                .push(" ORDER BY ")
                .append(list(self.order.into_iter().map(|o| o.0)));
        }
        if let Some(limit) = self.limit {
            expr = expr.push(" LIMIT ").append(limit);
        }
        if let Some(offset) = self.offset {
            expr = expr.push(" OFFSET ").append(offset);
        }
        if self.lock {
            expr = expr.push(" FOR UPDATE");
        }
        expr
    }
}
