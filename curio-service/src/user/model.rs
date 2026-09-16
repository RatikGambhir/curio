use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize)]
pub struct ConversationRequest {
    pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationResponse {
    pub id: String,
    pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationLookupResponse {
    pub id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveUserRequest {
    pub id: String,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, FromRow, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserRecord {
    pub id: String,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

impl UserRecord {
    /// The columns backing a user row.
    pub(crate) const COLUMNS: &'static str = "id, name, email, avatar_url, created_at, updated_at";
}

pub struct SaveUser<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub email: &'a str,
    pub avatar_url: Option<&'a str>,
}
