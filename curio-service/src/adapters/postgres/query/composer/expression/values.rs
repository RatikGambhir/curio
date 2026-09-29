//! Column enums, bound payloads, literals and tuple values.
use super::super::super::SqlColumn;
use super::{Expr, Part, list};
use sqlx::{Encode, Postgres, Type};

/// An unqualified column from a feature-owned `SqlColumn` enum.
pub fn col<'a>(column: impl SqlColumn) -> Expr<'a> {
    identifier(column.sql())
}
/// `alias.column`; prefer `SqlColumn::of`.
pub(in super::super::super) fn qualified<'a>(
    alias: &'static str,
    column: &'static str,
) -> Expr<'a> {
    identifier(alias).push(".").append(identifier(column))
}
/// `*` projection.
pub fn all<'a>() -> Expr<'a> {
    Expr::sql("*")
}
/// `alias.*` projection.
pub fn all_of<'a>(alias: &'static str) -> Expr<'a> {
    identifier(alias).push(".*")
}
impl<'a, C: SqlColumn> From<C> for Expr<'a> {
    fn from(column: C) -> Self {
        col(column)
    }
}
/// Code-owned table, alias or CTE name.
pub(in super::super) fn identifier<'a>(name: &'static str) -> Expr<'a> {
    Expr {
        parts: vec![Part::Identifier(name)],
    }
}
/// Values are encoded by SQLx only when the complete statement is built.
pub fn val<'a, T>(value: T) -> Expr<'a>
where
    T: 'a + Send + Encode<'a, Postgres> + Type<Postgres>,
{
    Expr {
        parts: vec![Part::Bind(Box::new(move |query| {
            query.push_bind(value);
        }))],
    }
}
/// Code-owned numeric constants, such as recursive depths or rank increments.
pub fn int<'a>(value: i64) -> Expr<'a> {
    Expr {
        parts: vec![Part::Integer(value)],
    }
}
pub fn null<'a>() -> Expr<'a> {
    Expr::sql("NULL")
}
pub fn current_timestamp<'a>() -> Expr<'a> {
    Expr::sql("CURRENT_TIMESTAMP")
}
pub fn millisecond<'a>() -> Expr<'a> {
    Expr::sql("interval '1 millisecond'")
}
pub fn tuple<'a>(items: impl IntoIterator<Item = Expr<'a>>) -> Expr<'a> {
    list(items).grouped()
}
