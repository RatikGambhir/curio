//! The calendar's rules.
//!
//! Everything that decides whether a request is acceptable lives here: what a
//! valid event looks like, how wide a view may be, and what a storage failure
//! means to the caller. This layer knows nothing about HTTP — it takes and
//! returns plain types and a `CalendarError` — and nothing about SQL, which
//! keeps the rules readable without a database in front of you.

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

/// A local day is not always 24 hours, and a local week is not always 168: a
/// DST transition shifts the UTC bounds the client computes by an hour. The
/// caps below are span *limits*, so they carry enough slack to let a legitimate
/// request through while still refusing a decade.
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
        let id = match input.id.as_deref().map(str::trim) {
            Some(id) if !id.is_empty() => id.to_owned(),
            _ => uuid::Uuid::new_v4().to_string(),
        };

        let description = optional(input.description.as_deref());
        let status = optional(input.status.as_deref());
        let priority = optional(input.priority.as_deref());
        let start_date = input.start_date.trim();
        let end_date = optional(input.end_date.as_deref());

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

    /// One method serves day, week, month and agenda, because they are the same
    /// query with different bounds.
    ///
    /// The client sends the bounds rather than the service deriving them from a
    /// view name and an anchor: the boundaries depend on the caller's time zone
    /// and its week-start convention, and the grid already computes them for
    /// rendering. Deriving them a second time here would create a source of
    /// truth that can disagree with what the grid draws.
    ///
    /// `view` still earns its place by bounding the span, so one client bug
    /// cannot request a decade and scan the table.
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

/// Collapses a storage failure into a client-facing error.
///
/// The unique and foreign key violations are the two the caller can act on — in
/// particular a user only gets a `users` row once the profile wizard runs, so
/// someone who signs in and goes straight to the calendar would otherwise get a
/// `500`. Everything else is logged here and sanitized, because the cause
/// belongs to operators and the message belongs to the client.
fn storage_error(context: &str, error: sqlx::Error) -> CalendarError {
    if let sqlx::Error::Database(ref database_error) = error {
        if database_error.is_unique_violation() {
            return CalendarError::Conflict("An event with that id already exists.");
        }
        if database_error.is_foreign_key_violation() {
            return CalendarError::UnknownUser;
        }
    }

    // The service has no logging crate yet and main.rs uses println!, so this
    // matches rather than dropping the cause silently.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    /// The rules are reachable without a router, which is the point of keeping
    /// them out of the handlers. A real pool still backs the service so the
    /// storage-error mapping is exercised by the same tests.
    async fn service() -> CalendarService {
        let database = Database::connect("sqlite::memory:").await.unwrap();
        CalendarService::new(CalendarRepository::new(database.pool().clone()))
    }

    fn create_input(title: &str) -> CreateEventInput {
        CreateEventInput {
            id: None,
            user_id: "user-a".to_owned(),
            title: title.to_owned(),
            description: None,
            status: None,
            priority: None,
            all_day: false,
            start_date: "2026-06-22T14:00:00.000Z".to_owned(),
            end_date: None,
        }
    }

    fn range(view: CalendarView, start: &str, end: &str) -> ListEventsQuery {
        ListEventsQuery {
            user_id: "user-a".to_owned(),
            view,
            start: start.to_owned(),
            end: end.to_owned(),
        }
    }

    #[tokio::test]
    async fn a_blank_title_never_reaches_storage() {
        let error = service().await.create_event(create_input("   ")).await;

        assert_eq!(
            error,
            Err(CalendarError::Invalid("An event needs a title."))
        );
    }

    #[tokio::test]
    async fn an_unknown_status_is_rejected_before_the_insert() {
        let mut input = create_input("Design review");
        input.status = Some("procrastinating".to_owned());

        assert_eq!(
            service().await.create_event(input).await,
            Err(CalendarError::Invalid("That is not a known event status."))
        );
    }

    #[tokio::test]
    async fn an_event_for_a_user_without_a_profile_is_an_unknown_user() {
        // Nothing created a users row, so the foreign key is what rejects this.
        let error = service()
            .await
            .create_event(create_input("Design review"))
            .await;

        assert_eq!(error, Err(CalendarError::UnknownUser));
    }

    #[tokio::test]
    async fn a_range_wider_than_its_view_is_rejected() {
        let error = service()
            .await
            .events_in_range(range(
                CalendarView::Day,
                "2026-06-01T00:00:00.000Z",
                "2026-07-01T00:00:00.000Z",
            ))
            .await;

        assert_eq!(
            error,
            Err(CalendarError::Invalid(
                "That range is wider than the requested view can cover."
            ))
        );
    }

    #[tokio::test]
    async fn a_days_worth_of_dst_slack_is_still_a_day() {
        // A local day that crosses a DST boundary is 25 hours once converted to
        // UTC, and must not be mistaken for an oversized range.
        let events = service()
            .await
            .events_in_range(range(
                CalendarView::Day,
                "2026-11-01T07:00:00.000Z",
                "2026-11-02T08:00:00.000Z",
            ))
            .await;

        assert_eq!(events, Ok(Vec::new()));
    }

    #[tokio::test]
    async fn an_inverted_range_is_rejected() {
        let error = service()
            .await
            .events_in_range(range(
                CalendarView::Day,
                "2026-06-23T00:00:00.000Z",
                "2026-06-22T00:00:00.000Z",
            ))
            .await;

        assert_eq!(
            error,
            Err(CalendarError::Invalid(
                "The range end must fall after the range start."
            ))
        );
    }
}
