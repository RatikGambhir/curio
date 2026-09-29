//! Allowlisted SQL functions and CASE expressions.
use super::{Expr, list};

/// Code-owned SQL functions; a request can never choose a function name.
#[derive(Clone, Copy)]
pub enum Function {
    Count,
    Max,
    Coalesce,
    Greatest,
    Least,
    ClockTimestamp,
    RowNumber,
    Unnest,
    Sum,
    Sqrt,
    NullIf,
    Cardinality,
    WebsearchToTsquery,
    TsRankCd,
}
impl Function {
    fn sql(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::Max => "max",
            Self::Coalesce => "COALESCE",
            Self::Greatest => "GREATEST",
            Self::Least => "LEAST",
            Self::ClockTimestamp => "clock_timestamp",
            Self::RowNumber => "row_number",
            Self::Unnest => "unnest",
            Self::Sum => "SUM",
            Self::Sqrt => "SQRT",
            Self::NullIf => "NULLIF",
            Self::Cardinality => "cardinality",
            Self::WebsearchToTsquery => "websearch_to_tsquery",
            Self::TsRankCd => "ts_rank_cd",
        }
    }
}
pub fn call<'a>(function: Function, args: impl IntoIterator<Item = Expr<'a>>) -> Expr<'a> {
    Expr::sql(function.sql())
        .push("(")
        .append(list(args))
        .push(")")
}
pub fn case_when<'a>(
    branches: impl IntoIterator<Item = (Expr<'a>, Expr<'a>)>,
    otherwise: Expr<'a>,
) -> Expr<'a> {
    let mut expr = Expr::sql("CASE");
    for (condition, value) in branches {
        expr = expr
            .push(" WHEN ")
            .append(condition)
            .push(" THEN ")
            .append(value);
    }
    expr.push(" ELSE ").append(otherwise).push(" END").grouped()
}
