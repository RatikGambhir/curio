use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

#[derive(Debug, FromRow, Serialize, PartialEq, Eq)]
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
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
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
    pub starts_at: &'a DateTime<Utc>,
    pub ends_at: &'a DateTime<Utc>,
}
