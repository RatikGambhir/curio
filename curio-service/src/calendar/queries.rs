use axum::{
    Extension, Json,
    extract::{Query, State},
};
use chrono::Duration;
use serde::{Deserialize, Serialize};

use crate::{
    CurrentUser,
    calendar::{CalendarView, error::CalendarError, time},
    database::{CalendarEventRecord, Database},
};

/// A local day is not always 24 hours, and a local week is not always 168: a
/// DST transition shifts the UTC bounds the client computes by an hour. The
/// caps below are span *limits*, so they carry enough slack to let a legitimate
/// request through while still refusing a decade.
const DST_SLACK_HOURS: i64 = 2;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEventsParams {
    pub user_id: String,
    pub view: CalendarView,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Serialize)]
pub struct EventsResponse {
    pub events: Vec<CalendarEventRecord>,
}

/// One endpoint serves day, week, month and agenda, because they are the same
/// query with different bounds.
///
/// The client sends the bounds rather than the server deriving them from a view
/// name and an anchor: the boundaries depend on the caller's time zone and its
/// week-start convention, and the grid already computes them for rendering.
/// Deriving them a second time here would create a source of truth that can
/// disagree with what the grid draws.
///
/// `view` still earns its place by bounding the span, so one client bug cannot
/// request a decade and scan the table.
pub async fn list_events(
    State(database): State<Database>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(params): Query<ListEventsParams>,
) -> Result<Json<EventsResponse>, CalendarError> {
    let user_id = params.user_id.trim();
    if user_id.is_empty() {
        return Err(CalendarError::Invalid(
            "A calendar range needs the id of the user who owns it.",
        ));
    }

    let start = time::parse_bound(params.start.trim()).ok_or(CalendarError::Invalid(
        "The range start could not be parsed.",
    ))?;
    let end = time::parse_bound(params.end.trim())
        .ok_or(CalendarError::Invalid("The range end could not be parsed."))?;

    if end <= start {
        return Err(CalendarError::Invalid(
            "The range end must fall after the range start.",
        ));
    }
    if end - start > max_span(params.view) {
        return Err(CalendarError::Invalid(
            "That range is wider than the requested view can cover.",
        ));
    }

    let events = database
        .calendar_events_in_range(
            user_id,
            &time::format_instant(start),
            &time::format_instant(end),
        )
        .await
        .map_err(|error| CalendarError::from_storage("listing calendar events", error))?;

    Ok(Json(EventsResponse { events }))
}

/// A month grid runs to 42 days because it includes the leading and trailing
/// weeks around the month itself.
fn max_span(view: CalendarView) -> Duration {
    let days = match view {
        CalendarView::Day => 1,
        CalendarView::Week => 7,
        CalendarView::Month => 42,
        CalendarView::Agenda => 92,
    };

    Duration::days(days) + Duration::hours(DST_SLACK_HOURS)
}
