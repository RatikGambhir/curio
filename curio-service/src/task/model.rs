use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};
use sqlx::FromRow;

pub const TASK_STATUSES: [&str; 5] = ["scheduled", "in-progress", "blocked", "done", "cancelled"];
pub const TASK_PRIORITIES: [&str; 3] = ["low", "medium", "high"];
pub const DEFAULT_TASK_STATUS: &str = "scheduled";

#[derive(Debug, FromRow, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskRecord {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: Option<String>,
    pub active: bool,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub set_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_optional_timestamp")]
    pub due_at: Option<DateTime<Utc>>,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

impl TaskRecord {
    /// The columns backing a task row.
    pub(crate) const COLUMNS: &'static str = "id, user_id, name, description, status, priority, \
         active, set_at, due_at, created_at, updated_at";
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskInput {
    pub id: Option<String>,
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub active: Option<bool>,
    pub set_at: Option<String>,
    pub due_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTasksQuery {
    #[serde(default)]
    pub user_id: String,
}

pub struct NewTask<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub status: &'a str,
    pub priority: Option<&'a str>,
    pub active: bool,
    pub set_at: &'a DateTime<Utc>,
    pub due_at: Option<&'a DateTime<Utc>>,
}

fn serialize_optional_timestamp<S>(
    value: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => crate::database::serialize_timestamp(value, serializer),
        None => serializer.serialize_none(),
    }
}
