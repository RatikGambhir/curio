//! Chat wire contract and lifecycle values, independent of HTTP, SQLx and OpenAI.
use serde::{Deserialize, Serialize};

/// `POST /v1/chat/stream` body. The client owns all correlation IDs.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChatStreamRequest {
    pub conversation_id: String,
    pub user_message_id: String,
    pub assistant_message_id: String,
    pub prompt: String,
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

/// Public SSE events. The variant selects the SSE `event:` name; the payload is
/// serialized untagged as the `data:` line.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(untagged)]
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

    pub fn is_terminal(&self) -> bool {
        !matches!(self, Self::Token(_))
    }

    pub fn data_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn encode(&self) -> Result<String, serde_json::Error> {
        Ok(format!(
            "event: {}\ndata: {}\n\n",
            self.event_name(),
            self.data_json()?
        ))
    }
}

/// Terminal assistant state. Impossible combinations (such as a failed response
/// with a completion ID) cannot be passed to persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssistantOutcome<'a> {
    Completed { response_id: &'a str },
    Failed { code: &'a str },
    Interrupted { code: &'a str },
}

/// Storage adapters log classified diagnostics before returning this opaque error.
#[derive(Debug, Clone, Copy)]
pub struct ChatStorageError;

#[cfg(test)]
#[path = "../../../tests/unit/domains/chat/model.rs"]
mod tests;
