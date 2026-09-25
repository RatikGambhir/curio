use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::{
    domains::calendar::{
        Service,
        model::{CalendarError, CalendarEvent, CreateEventInput, ListEventsQuery},
    },
    shared::auth::CurrentUser,
};

#[derive(Debug, Serialize)]
pub struct EventsResponse {
    events: Vec<CalendarEvent>,
}

pub async fn create_event(
    State(service): State<Arc<Service>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(input): Json<CreateEventInput>,
) -> Result<(StatusCode, Json<CalendarEvent>), CalendarError> {
    authorize_owner(&current_user, &input.user_id)?;
    let event = service.create_event(input).await?;
    Ok((StatusCode::CREATED, Json(event)))
}

pub async fn list_events(
    State(service): State<Arc<Service>>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ListEventsQuery>,
) -> Result<Json<EventsResponse>, CalendarError> {
    authorize_owner(&current_user, &query.user_id)?;
    let events = service.events_in_range(query).await?;
    Ok(Json(EventsResponse { events }))
}

fn authorize_owner(
    current_user: &CurrentUser,
    requested_user_id: &str,
) -> Result<(), CalendarError> {
    // Leave required-field validation to the service so a blank user id keeps
    // returning a validation error rather than being mistaken for an authz failure.
    if requested_user_id.trim().is_empty() || current_user.owns(requested_user_id) {
        Ok(())
    } else {
        Err(CalendarError::Forbidden)
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

impl IntoResponse for CalendarError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            Self::Invalid(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "Calendar events can only be accessed by their owner.",
            ),
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
            Self::UnknownUser => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "No profile exists for that user yet.",
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "The calendar is unavailable.",
            ),
        };

        (status, Json(ErrorBody { error })).into_response()
    }
}
