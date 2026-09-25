//! HTTP/SSE adapters for chat use cases.
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{
        IntoResponse, Sse,
        sse::{Event, KeepAlive},
    },
};
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{Instrument, info_span};

use super::{
    Service,
    domain::{ChatEvent, ConversationRecord, MessageRecord},
    protocol::{ChatStreamEvent, ChatStreamRequest},
    service::{ChatEventSink, DeliveryError},
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationsResponse {
    conversations: Vec<ConversationRecord>,
}
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessagesResponse {
    conversation_id: String,
    messages: Vec<MessageRecord>,
}

struct SseSink {
    sender: mpsc::Sender<Result<Event, Infallible>>,
    request: ChatStreamRequest,
}
impl ChatEventSink for SseSink {
    async fn send(&self, event: ChatEvent) -> Result<(), DeliveryError> {
        let event = match event {
            ChatEvent::Token(token) => ChatStreamEvent::token(&self.request, token),
            ChatEvent::Done(response_id) => ChatStreamEvent::done(&self.request, response_id),
            ChatEvent::Error { code, message } => {
                ChatStreamEvent::error(&self.request, code, message)
            }
        }
        .into_axum_event()
        .map_err(|_| DeliveryError::Serialization)?;
        self.sender
            .send(Ok(event))
            .await
            .map_err(|_| DeliveryError::Disconnected)
    }
}

pub async fn stream_chat(
    State(service): State<Service>,
    Json(request): Json<ChatStreamRequest>,
) -> impl IntoResponse {
    let (sender, receiver) = mpsc::channel(32);
    let span = info_span!("chat_stream", conversation_id = ?request.conversation_id, user_message_id = ?request.user_message_id, assistant_message_id = ?request.assistant_message_id);
    let sink = SseSink {
        sender,
        request: request.clone(),
    };
    match service.start(request).instrument(span.clone()).await {
        Ok(session) => {
            tokio::spawn(
                async move {
                    service.run(session, &sink).await;
                }
                .instrument(span),
            );
        }
        Err(event) => {
            let _ = sink.send(event).await;
        }
    }
    Sse::new(ReceiverStream::new(receiver)).keep_alive(KeepAlive::default())
}

pub async fn list_conversations(
    State(service): State<Service>,
) -> Result<Json<ConversationsResponse>, StatusCode> {
    service
        .list_conversations()
        .await
        .map(|conversations| Json(ConversationsResponse { conversations }))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn conversation_messages(
    State(service): State<Service>,
    Path(conversation_id): Path<String>,
) -> Result<Json<ConversationMessagesResponse>, StatusCode> {
    let span = info_span!("conversation_messages", conversation_id = ?conversation_id);
    let messages = service
        .conversation_messages(&conversation_id)
        .instrument(span)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ConversationMessagesResponse {
        conversation_id,
        messages,
    }))
}
