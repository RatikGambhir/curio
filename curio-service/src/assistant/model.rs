use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChatStreamRequest {
    pub conversation_id: String,
    pub user_message_id: String,
    pub assistant_message_id: String,
    pub prompt: String,
}

#[derive(Debug, FromRow, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConversationRecord {
    pub id: String,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MessageRecord {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub status: String,
    pub response_id: Option<String>,
    pub error_code: Option<String>,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

impl ConversationRecord {
    /// The columns backing a conversation row.
    pub(crate) const COLUMNS: &'static str = "id, created_at, updated_at";
}

impl MessageRecord {
    /// The columns backing a message row. The `sort_order` tiebreaker is an
    /// ordering detail and is deliberately not selected.
    pub(crate) const COLUMNS: &'static str = "id, conversation_id, role, content, status, \
         response_id, error_code, created_at, updated_at";
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationsResponse {
    pub conversations: Vec<ConversationRecord>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessagesResponse {
    pub conversation_id: String,
    pub messages: Vec<MessageRecord>,
}

pub struct EncodedChatEvent {
    pub name: &'static str,
    pub data: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TokenEvent {
    pub conversation_id: String,
    pub message_id: String,
    pub token: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEvent {
    pub conversation_id: String,
    pub message_id: String,
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DoneEvent {
    pub conversation_id: String,
    pub message_id: String,
    pub response_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChatStreamEvent {
    Token(TokenEvent),
    Error(ErrorEvent),
    Done(DoneEvent),
}

impl ChatStreamEvent {
    pub fn token(request: &ChatStreamRequest, token: impl Into<String>) -> Self {
        Self::Token(TokenEvent {
            conversation_id: request.conversation_id.clone(),
            message_id: request.assistant_message_id.clone(),
            token: token.into(),
        })
    }

    pub fn error(
        request: &ChatStreamRequest,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Error(ErrorEvent {
            conversation_id: request.conversation_id.clone(),
            message_id: request.assistant_message_id.clone(),
            code: code.into(),
            message: message.into(),
        })
    }

    pub fn done(request: &ChatStreamRequest, response_id: impl Into<String>) -> Self {
        Self::Done(DoneEvent {
            conversation_id: request.conversation_id.clone(),
            message_id: request.assistant_message_id.clone(),
            response_id: response_id.into(),
        })
    }

    pub fn event_name(&self) -> &'static str {
        match self {
            Self::Token(_) => "token",
            Self::Error(_) => "error",
            Self::Done(_) => "done",
        }
    }

    pub fn data_json(&self) -> Result<String, serde_json::Error> {
        match self {
            Self::Token(payload) => serde_json::to_string(payload),
            Self::Error(payload) => serde_json::to_string(payload),
            Self::Done(payload) => serde_json::to_string(payload),
        }
    }

    #[cfg(test)]
    pub fn encode(&self) -> Result<String, serde_json::Error> {
        Ok(format!(
            "event: {}\ndata: {}\n\n",
            self.event_name(),
            self.data_json()?
        ))
    }

    pub fn into_encoded(self) -> Result<EncodedChatEvent, serde_json::Error> {
        Ok(EncodedChatEvent {
            name: self.event_name(),
            data: self.data_json()?,
        })
    }
}

#[cfg(test)]
#[path = "../../tests/unit/assistant/model.rs"]
mod tests;
