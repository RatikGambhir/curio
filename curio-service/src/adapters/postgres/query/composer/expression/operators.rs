//! Comparisons, boolean groups, arithmetic, casts, aliases and ordering.
use super::super::super::Direction;
use super::super::super::SqlColumn;
use super::super::{Order, Select};
use super::{Expr, SqlType, identifier};

impl<'a> Expr<'a> {
    fn binary(self, operator: &'static str, rhs: Self) -> Self {
        self.push(operator).append(rhs).grouped()
    }
    pub fn eq(self, rhs: Self) -> Self {
        self.binary(" = ", rhs)
    }
    pub fn ne(self, rhs: Self) -> Self {
        self.binary(" <> ", rhs)
    }
    pub fn lt(self, rhs: Self) -> Self {
        self.binary(" < ", rhs)
    }
    pub fn le(self, rhs: Self) -> Self {
        self.binary(" <= ", rhs)
    }
    pub fn gt(self, rhs: Self) -> Self {
        self.binary(" > ", rhs)
    }
    pub fn ge(self, rhs: Self) -> Self {
        self.binary(" >= ", rhs)
    }
    pub fn and(self, rhs: Self) -> Self {
        self.binary(" AND ", rhs)
    }
    pub fn or(self, rhs: Self) -> Self {
        self.binary(" OR ", rhs)
    }
    pub fn plus(self, rhs: Self) -> Self {
        self.binary(" + ", rhs)
    }
    pub fn minus(self, rhs: Self) -> Self {
        self.binary(" - ", rhs)
    }
    pub fn times(self, rhs: Self) -> Self {
        self.binary(" * ", rhs)
    }
    pub fn divided_by(self, rhs: Self) -> Self {
        self.binary(" / ", rhs)
    }
    pub fn distinct_from(self, rhs: Self) -> Self {
        self.binary(" IS DISTINCT FROM ", rhs)
    }
    pub fn not_distinct_from(self, rhs: Self) -> Self {
        self.binary(" IS NOT DISTINCT FROM ", rhs)
    }
    pub fn matches_text(self, rhs: Self) -> Self {
        self.binary(" @@ ", rhs)
    }
    pub fn is_null(self) -> Self {
        self.push(" IS NULL").grouped()
    }
    pub fn is_not_null(self) -> Self {
        self.push(" IS NOT NULL").grouped()
    }
    pub fn eq_any(self, values: Self) -> Self {
        self.push(" = ANY(").append(values).push(")").grouped()
    }
    pub fn in_query(self, query: Select<'a>) -> Self {
        self.push(" IN ").append(query.scalar()).grouped()
    }
    pub fn cast(self, sql_type: SqlType) -> Self {
        self.grouped().push("::").push(sql_type.sql())
    }
    /// Names a projected expression with a column enum.
    pub fn alias(self, name: impl SqlColumn) -> Self {
        self.alias_as(name.sql())
    }
    pub(in super::super) fn alias_as(self, name: &'static str) -> Self {
        self.push(" AS ").append(identifier(name))
    }
    pub fn asc(self) -> Order<'a> {
        Order(self.push(" ASC"))
    }
    pub fn desc(self) -> Order<'a> {
        Order(self.push(" DESC"))
    }
    pub fn order(self, direction: Direction) -> Order<'a> {
        match direction {
            Direction::Ascending => self.asc(),
            Direction::Descending => self.desc(),
        }
    }
}
