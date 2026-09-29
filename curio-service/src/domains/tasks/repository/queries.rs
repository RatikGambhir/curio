//! Shared owner, created-page and audit timestamp query expressions.
use super::super::{
    filter::{CreatedPage, cursor_time},
    model::TaskError,
};
use super::columns::TaskColumn;
use crate::adapters::postgres::query::{
    Expr, Function, Select, SqlType, call, col, int, millisecond, tuple, val,
};

pub(super) fn owned_task<'a>(owner: &'a str, id: &'a str) -> Select<'a> {
    Select::from("tasks")
        .columns([int(1)])
        .where_(col(TaskColumn::OwnerId).eq(val(owner)))
        .where_(col(TaskColumn::Id).eq(val(id)))
}

/// Newest-first keyset page over a creation timestamp and id column.
pub(super) fn created_page<'a>(
    mut query: Select<'a>,
    page: &'a CreatedPage,
    created: impl Fn() -> Expr<'a>,
    id: impl Fn() -> Expr<'a>,
) -> Result<Select<'a>, TaskError> {
    if let Some(cursor) = &page.cursor {
        query = query.where_(
            tuple([created(), id()])
                .lt(tuple([val(cursor_time(cursor.created)?), val(&cursor.id)])),
        );
    }
    Ok(query
        .order_by(created().desc())
        .order_by(id().desc())
        .limit(val((page.limit + 1) as i64)))
}

/// A strictly increasing millisecond audit timestamp after `updated_at`.
pub(super) fn advance(updated_at: Expr<'_>) -> Expr<'_> {
    call(
        Function::Greatest,
        [
            call(Function::ClockTimestamp, []).cast(SqlType::TimestampMillis),
            updated_at.plus(millisecond()),
        ],
    )
}
