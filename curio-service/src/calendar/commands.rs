use axum::{Extension, Json, extract::State, http::StatusCode};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    CurrentUser,
    calendar::{
        EVENT_PRIORITIES, EVENT_STATUSES,
        error::CalendarError,
        time::{self},
    },
    database::{CalendarEventRecord, Database},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventRequest {
    /// Client-generated when present, which is what makes a retried create
    /// idempotent instead of duplicating the event.
    pub id: Option<String>,
    pub user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    #[serde(default)]
    pub all_day: bool,
    pub start_date: String,
    pub end_date: Option<String>,
}

/// `Extension<CurrentUser>` is the slot real identity lands in. Today
/// `authorize_current_user` accepts any non-empty bearer token without deriving
/// who sent it, so `userId` travels in the payload and this is ignored — the
/// same shape `user/commands.rs` already uses. When auth lands the handler
/// reads `current_user.id` and the signature does not change.
pub async fn create_event(
    State(database): State<Database>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(request): Json<CreateEventRequest>,
) -> Result<(StatusCode, Json<CalendarEventRecord>), CalendarError> {
    let user_id = required(
        &request.user_id,
        "An event needs the id of the user who owns it.",
    )?;
    let title = required(&request.title, "An event needs a title.")?;
    let id = match request.id.as_deref().map(str::trim) {
        Some(id) if !id.is_empty() => id.to_owned(),
        _ => Uuid::new_v4().to_string(),
    };

    let description = optional(request.description.as_deref());
    let status = optional(request.status.as_deref());
    let priority = optional(request.priority.as_deref());
    let start_date = request.start_date.trim();
    let end_date = optional(request.end_date.as_deref());

    if let Some(status) = status
        && !EVENT_STATUSES.contains(&status)
    {
        return Err(CalendarError::Invalid("That is not a known event status."));
    }
    if let Some(priority) = priority
        && !EVENT_PRIORITIES.contains(&priority)
    {
        return Err(CalendarError::Invalid(
            "That is not a known event priority.",
        ));
    }

    let interval = time::normalize_event(request.all_day, start_date, end_date)
        .map_err(CalendarError::Invalid)?;

    let event = database
        .create_calendar_event(crate::database::NewCalendarEvent {
            id: &id,
            user_id,
            title,
            description,
            status,
            priority,
            all_day: request.all_day,
            start_date,
            end_date,
            starts_at: &interval.starts_at,
            ends_at: &interval.ends_at,
        })
        .await
        .map_err(|error| CalendarError::from_storage("creating a calendar event", error))?;

    Ok((StatusCode::CREATED, Json(event)))
}

fn required<'a>(value: &'a str, message: &'static str) -> Result<&'a str, CalendarError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(CalendarError::Invalid(message));
    }
    Ok(value)
}

fn optional(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}
