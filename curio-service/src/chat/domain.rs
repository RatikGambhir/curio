//! Conversation records and assistant lifecycle values, independent of adapters.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChatTurn {
    pub conversation_id: String,
    pub user_message_id: String,
    pub assistant_message_id: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConversationRecord {
    pub id: String,
    #[serde(serialize_with = "crate::serialization::serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "crate::serialization::serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MessageRecord {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub status: String,
    pub response_id: Option<String>,
    pub error_code: Option<String>,
    #[serde(serialize_with = "crate::serialization::serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "crate::serialization::serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

/// Terminal assistant state. Impossible combinations (such as a failed response
/// with a completion ID) cannot be passed to persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssistantOutcome<'a> {
    Completed { response_id: &'a str },
    Failed { code: &'a str },
    Interrupted { code: &'a str },
}

/// Provider-independent deltas consumed by the chat application service.
#[derive(Debug, PartialEq, Eq)]
pub enum ModelEvent {
    Token(String),
    Done(String),
    Error { code: &'static str, message: String },
}

/// Application events mapped to the public SSE envelope by the HTTP adapter.
#[derive(Debug, PartialEq, Eq)]
pub enum ChatEvent {
    Token(String),
    Done(String),
    Error { code: &'static str, message: String },
}
impl ChatEvent {
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self::Error {
            code,
            message: message.into(),
        }
    }
    pub fn is_terminal(&self) -> bool {
        !matches!(self, Self::Token(_))
    }
}

/// Storage adapters log classified diagnostics before returning this opaque error.
#[derive(Debug, Clone, Copy)]
pub struct ChatStorageError;
