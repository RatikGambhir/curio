use chrono::Duration;

use crate::calendar::{
    error::CalendarError,
    models::{
        CalendarEvent, CalendarView, CreateEventInput, EVENT_PRIORITIES, EVENT_STATUSES,
        ListEventsQuery, NewCalendarEvent,
    },
    repository::CalendarRepository,
    time,
};

const DST_SLACK_HOURS: i64 = 2;

#[derive(Clone)]
pub struct CalendarService {
    repository: CalendarRepository,
}

impl CalendarService {
    pub fn new(repository: CalendarRepository) -> Self {
        Self { repository }
    }

    pub async fn create_event(
        &self,
        input: CreateEventInput,
    ) -> Result<CalendarEvent, CalendarError> {
        let user_id = required(
            &input.user_id,
            "An event needs the id of the user who owns it.",
        )?;
        let title = required(&input.title, "An event needs a title.")?;
        let id = input
            .id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let description = optional(input.description.as_deref());
        let status = optional(input.status.as_deref());
        let priority = optional(input.priority.as_deref());
        let start_date = input.start_date.trim();
        let end_date = optional(input.end_date.as_deref());

        if status.is_some_and(|value| !EVENT_STATUSES.contains(&value)) {
            return Err(CalendarError::Invalid("That is not a known event status."));
        }
        if priority.is_some_and(|value| !EVENT_PRIORITIES.contains(&value)) {
            return Err(CalendarError::Invalid(
                "That is not a known event priority.",
            ));
        }

        let interval = time::normalize_event(input.all_day, start_date, end_date)
            .map_err(CalendarError::Invalid)?;

        self.repository
            .insert(NewCalendarEvent {
                id: &id,
                user_id,
                title,
                description,
                status,
                priority,
                all_day: input.all_day,
                start_date,
                end_date,
                starts_at: &interval.starts_at,
                ends_at: &interval.ends_at,
            })
            .await
            .map_err(|error| storage_error("creating a calendar event", error))
    }

    pub async fn events_in_range(
        &self,
        query: ListEventsQuery,
    ) -> Result<Vec<CalendarEvent>, CalendarError> {
        let user_id = required(
            &query.user_id,
            "A calendar range needs the id of the user who owns it.",
        )?;
        let start = time::parse_bound(query.start.trim()).ok_or(CalendarError::Invalid(
            "The range start could not be parsed.",
        ))?;
        let end = time::parse_bound(query.end.trim())
            .ok_or(CalendarError::Invalid("The range end could not be parsed."))?;

        if end <= start {
            return Err(CalendarError::Invalid(
                "The range end must fall after the range start.",
            ));
        }
        if end - start > max_span(query.view) {
            return Err(CalendarError::Invalid(
                "That range is wider than the requested view can cover.",
            ));
        }

        self.repository
            .in_range(
                user_id,
                &time::format_instant(start),
                &time::format_instant(end),
            )
            .await
            .map_err(|error| storage_error("listing calendar events", error))
    }
}

fn max_span(view: CalendarView) -> Duration {
    let days = match view {
        CalendarView::Day => 1,
        CalendarView::Week => 7,
        CalendarView::Month => 42,
        CalendarView::Agenda => 92,
    };

    Duration::days(days) + Duration::hours(DST_SLACK_HOURS)
}

fn storage_error(context: &str, error: sqlx::Error) -> CalendarError {
    if let sqlx::Error::Database(ref database_error) = error {
        if database_error.is_unique_violation() {
            return CalendarError::Conflict("An event with that id already exists.");
        }
        if database_error.is_foreign_key_violation() {
            return CalendarError::UnknownUser;
        }
    }

    eprintln!("curio-service: {context} failed: {error}");
    CalendarError::Internal
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
