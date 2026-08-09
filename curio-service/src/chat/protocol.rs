use axum::response::sse::Event;
use serde::{Deserialize, Serialize};

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

    pub fn encode(&self) -> Result<String, serde_json::Error> {
        Ok(format!(
            "event: {}\ndata: {}\n\n",
            self.event_name(),
            self.data_json()?
        ))
    }

    pub fn into_axum_event(self) -> Result<Event, axum::Error> {
        let event = match self {
            Self::Token(payload) => Event::default().event("token").json_data(payload),
            Self::Error(payload) => Event::default().event("error").json_data(payload),
            Self::Done(payload) => Event::default().event("done").json_data(payload),
        }?;

        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ChatStreamRequest {
        ChatStreamRequest {
            conversation_id: "conversation-1".to_owned(),
            user_message_id: "user-1".to_owned(),
            assistant_message_id: "assistant-1".to_owned(),
            prompt: "Hello".to_owned(),
        }
    }

    #[test]
    fn request_uses_the_public_camel_case_contract() {
        let decoded: ChatStreamRequest = serde_json::from_str(
            r#"{"conversationId":"conversation-1","userMessageId":"user-1","assistantMessageId":"assistant-1","prompt":"Hello"}"#,
        )
        .unwrap();

        assert_eq!(decoded, request());
    }

    #[test]
    fn token_event_matches_the_exact_sse_contract() {
        let event = ChatStreamEvent::token(&request(), "Hello");

        assert_eq!(
            event.encode().unwrap(),
            "event: token\ndata: {\"conversationId\":\"conversation-1\",\"messageId\":\"assistant-1\",\"token\":\"Hello\"}\n\n"
        );
    }

    #[test]
    fn error_event_matches_the_exact_sse_contract() {
        let event = ChatStreamEvent::error(&request(), "provider_error", "Request failed");

        assert_eq!(
            event.encode().unwrap(),
            "event: error\ndata: {\"conversationId\":\"conversation-1\",\"messageId\":\"assistant-1\",\"code\":\"provider_error\",\"message\":\"Request failed\"}\n\n"
        );
    }

    #[test]
    fn done_event_matches_the_exact_sse_contract() {
        let event = ChatStreamEvent::done(&request(), "response-1");

        assert_eq!(
            event.encode().unwrap(),
            "event: done\ndata: {\"conversationId\":\"conversation-1\",\"messageId\":\"assistant-1\",\"responseId\":\"response-1\"}\n\n"
        );
    }
}
