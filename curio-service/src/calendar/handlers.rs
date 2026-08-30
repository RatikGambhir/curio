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
    events: Vec<CalendarEvent>,
}

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
