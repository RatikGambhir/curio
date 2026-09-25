//! Calendar values and validation, independent of HTTP and persistence.

use super::time;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

const DST_SLACK_HOURS: i64 = 2;

pub const EVENT_STATUSES: [&str; 5] = ["scheduled", "in-progress", "blocked", "done", "cancelled"];
pub const EVENT_PRIORITIES: [&str; 3] = ["low", "medium", "high"];

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CalendarView {
    Day,
    Week,
    Month,
    Agenda,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
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
    #[serde(serialize_with = "crate::shared::serialization::serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "crate::shared::serialization::serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEventInput {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEventsQuery {
    pub user_id: String,
    pub view: CalendarView,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalendarError {
    Invalid(&'static str),
    Forbidden,
    Conflict(&'static str),
    UnknownUser,
    Internal,
}

/// A normalized event that can only be constructed through domain validation.
pub struct NewCalendarEvent<'a> {
    id: String,
    user_id: &'a str,
    title: &'a str,
    description: Option<&'a str>,
    status: Option<&'a str>,
    priority: Option<&'a str>,
    all_day: bool,
    start_date: &'a str,
    end_date: Option<&'a str>,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
}

impl<'a> NewCalendarEvent<'a> {
    pub fn parse(input: &'a CreateEventInput) -> Result<Self, CalendarError> {
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

        Ok(Self {
            id,
            user_id,
            title,
            description,
            status,
            priority,
            all_day: input.all_day,
            start_date,
            end_date,
            starts_at: interval.starts_at,
            ends_at: interval.ends_at,
        })
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn user_id(&self) -> &'a str {
        self.user_id
    }
    pub fn title(&self) -> &'a str {
        self.title
    }
    pub fn description(&self) -> Option<&'a str> {
        self.description
    }
    pub fn status(&self) -> Option<&'a str> {
        self.status
    }
    pub fn priority(&self) -> Option<&'a str> {
        self.priority
    }
    pub fn all_day(&self) -> bool {
        self.all_day
    }
    pub fn start_date(&self) -> &'a str {
        self.start_date
    }
    pub fn end_date(&self) -> Option<&'a str> {
        self.end_date
    }
    pub fn starts_at(&self) -> DateTime<Utc> {
        self.starts_at
    }
    pub fn ends_at(&self) -> DateTime<Utc> {
        self.ends_at
    }
}

/// Owner-scoped, bounded half-open range used by calendar reads.
pub struct EventRange<'a> {
    user_id: &'a str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}
impl<'a> EventRange<'a> {
    pub fn parse(query: &'a ListEventsQuery) -> Result<Self, CalendarError> {
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

        Ok(Self {
            user_id,
            start,
            end,
        })
    }
    pub fn user_id(&self) -> &str {
        self.user_id
    }
    pub fn start(&self) -> DateTime<Utc> {
        self.start
    }
    pub fn end(&self) -> DateTime<Utc> {
        self.end
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
