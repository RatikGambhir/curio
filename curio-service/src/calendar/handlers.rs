//! The HTTP boundary. Extractors in, status codes and JSON out; every rule this
//! layer might be tempted to apply lives in the service instead.

use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::Serialize;

use crate::{
    CurrentUser,
    calendar::{
        error::CalendarError,
        models::{CalendarEvent, CreateEventInput, ListEventsQuery},
        service::CalendarService,
    },
};

#[derive(Debug, Serialize)]
pub struct EventsResponse {
    pub events: Vec<CalendarEvent>,
}

/// `Extension<CurrentUser>` is the slot real identity lands in. Today
/// `authorize_current_user` accepts any non-empty bearer token without deriving
/// who sent it, so `userId` travels in the payload and this is ignored — the
/// same shape `user/commands.rs` already uses. When auth lands the handler
/// reads `current_user.id` and the signature does not change.
///
/// Body-consuming extractors come last, since only one of them can run.
pub async fn create_event(
    State(service): State<CalendarService>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(input): Json<CreateEventInput>,
) -> Result<(StatusCode, Json<CalendarEvent>), CalendarError> {
    let event = service.create_event(input).await?;

    Ok((StatusCode::CREATED, Json(event)))
}

pub async fn list_events(
    State(service): State<CalendarService>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ListEventsQuery>,
) -> Result<Json<EventsResponse>, CalendarError> {
    let events = service.events_in_range(query).await?;

    Ok(Json(EventsResponse { events }))
}
