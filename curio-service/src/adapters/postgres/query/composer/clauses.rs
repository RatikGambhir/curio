//! Shared CTE, predicate, assignment and RETURNING clause assembly.
use super::expression::{Expr, identifier, invalid, list};

pub(super) fn where_clause<'a>(mut expr: Expr<'a>, filters: Vec<Expr<'a>>) -> Expr<'a> {
    for (index, filter) in filters.into_iter().enumerate() {
        expr = expr
            .push(if index == 0 { " WHERE " } else { " AND " })
            .append(filter);
    }
    expr
}
pub(super) fn cte_prefix<'a>(ctes: Vec<(&'static str, Expr<'a>)>, recursive: bool) -> Expr<'a> {
    if ctes.is_empty() {
        return Expr::sql("");
    }
    let mut expr = Expr::sql(if recursive {
        "WITH RECURSIVE "
    } else {
        "WITH "
    });
    expr = expr.append(list(ctes.into_iter().map(|(name, query)| {
        identifier(name).push(" AS ").append(query.grouped())
    })));
    expr.push(" ")
}

pub(super) fn validate_columns<'a>(mut expr: Expr<'a>, columns: &[&'static str]) -> Expr<'a> {
    if columns.is_empty() {
        expr = expr.append(invalid("a write needs at least one column"));
    }
    for (index, column) in columns.iter().enumerate() {
        if columns[..index].contains(column) {
            expr = expr.append(invalid("duplicate write column"));
        }
    }
    expr
}
pub(super) fn assignment_list<'a>(assignments: Vec<(&'static str, Expr<'a>)>) -> Expr<'a> {
    let columns: Vec<_> = assignments.iter().map(|(column, _)| *column).collect();
    let expr = list(
        assignments
            .into_iter()
            .map(|(column, expr)| identifier(column).push(" = ").append(expr)),
    );
    validate_columns(expr, &columns)
}
pub(super) fn returning<'a>(expr: Expr<'a>, columns: Vec<Expr<'a>>) -> Expr<'a> {
    if columns.is_empty() {
        expr
    } else {
        expr.push(" RETURNING ").append(list(columns))
    }
}
