//! The types that cross the calendar's layers.
//!
//! Handlers deserialize into the input types, the service validates them into
//! a [`NewCalendarEvent`], and the repository turns rows back into a
//! [`CalendarEvent`].

use serde::{Deserialize, Serialize};

/// The status and priority vocabularies the calendar UI uses. Validated in the
/// service so the repository never stores a value the client cannot render.
pub const EVENT_STATUSES: [&str; 4] = ["scheduled", "confirmed", "tentative", "cancelled"];
pub const EVENT_PRIORITIES: [&str; 3] = ["low", "medium", "high"];

/// The views the client can ask for.
///
/// Deserializing `view` as an enum means an unknown value fails in the `Query`
/// extractor and returns `400` before the handler runs, so neither the handler
/// nor the service ever sees one.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CalendarView {
    Day,
    Week,
    Month,
    Agenda,
}

/// A stored calendar event as the client sees it.
///
/// Deliberately exposes only the wire fields. The normalized `starts_at` /
/// `ends_at` instants exist to make range queries correct and are never
/// returned: the frontend classifies events by the *format* of `startDate`, so
/// handing it a normalized instant would turn every all-day event into a
/// midnight block.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub all_day: bool,
    pub start_date: String,
    pub end_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// An unvalidated create payload.
///
/// The wire shape and the service's input are the same shape, so this is one
/// type rather than a DTO copied field-for-field into a domain command. It
/// lives here rather than in `handlers` so the service does not have to depend
/// on the HTTP layer to name its own argument.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventInput {
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

/// An unvalidated range request. `start` and `end` arrive as strings and are
/// parsed in the service, so an unparseable bound is a domain error rather than
/// an extractor rejection.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEventsQuery {
    pub user_id: String,
    pub view: CalendarView,
    pub start: String,
    pub end: String,
}

/// A validated event on its way into storage, with the wire strings and the
/// normalized instants derived from them travelling together.
///
/// The service is the only thing that builds one of these, which is what makes
/// "every stored row was normalized" a property of the code rather than a
/// convention.
pub struct NewCalendarEvent<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub status: Option<&'a str>,
    pub priority: Option<&'a str>,
    pub all_day: bool,
    pub start_date: &'a str,
    pub end_date: Option<&'a str>,
    pub starts_at: &'a str,
    pub ends_at: &'a str,
}
