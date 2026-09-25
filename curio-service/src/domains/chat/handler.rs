//! HTTP/SSE adapters for chat use cases.
use std::{convert::Infallible, sync::Arc, time::Duration};

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::sse::{Event, KeepAlive, Sse},
};
use futures_util::stream::{self, Stream};
use serde::Serialize;
use tokio::sync::mpsc;
use tracing::{Instrument, info_span};

use super::{
    model::{ChatStreamEvent, ChatStreamRequest},
    service::{ChatService, ConversationRecord, MessageRecord},
};

#[derive(Clone)]
pub(super) struct ChatHttpState {
    pub chat: Arc<ChatService>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ConversationsResponse {
    conversations: Vec<ConversationRecord>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ConversationMessagesResponse {
    conversation_id: String,
    messages: Vec<MessageRecord>,
}

pub(super) async fn stream_chat_handler(
    State(state): State<ChatHttpState>,
    Json(request): Json<ChatStreamRequest>,
) -> (
    HeaderMap,
    Sse<impl Stream<Item = Result<Event, Infallible>>>,
) {
    let span = info_span!(
        "chat_stream",
        conversation_id = ?request.conversation_id,
        user_message_id = ?request.user_message_id,
        assistant_message_id = ?request.assistant_message_id,
    );
    let fallback = ChatStreamEvent::error(
        &request,
        "serialization_error",
        "The response event could not be encoded.",
    );
    let receiver = state.chat.ask(request).instrument(span).await;
    chat_sse(receiver, fallback)
}

pub(super) async fn list_conversations_handler(
    State(state): State<ChatHttpState>,
) -> Result<Json<ConversationsResponse>, StatusCode> {
    state
        .chat
        .list_conversations()
        .await
        .map(|conversations| Json(ConversationsResponse { conversations }))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub(super) async fn conversation_messages_handler(
    State(state): State<ChatHttpState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<ConversationMessagesResponse>, StatusCode> {
    let span = info_span!("conversation_messages", conversation_id = ?conversation_id);
    let messages = state
        .chat
        .conversation_messages(&conversation_id)
        .instrument(span)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ConversationMessagesResponse {
        conversation_id,
        messages,
    }))
}

/// Encodes service events as SSE. An event that cannot be encoded is replaced
/// by a correlated `error` event and ends the stream, which the service observes
/// as a disconnect and records as interrupted.
fn chat_sse(
    receiver: mpsc::Receiver<ChatStreamEvent>,
    fallback: ChatStreamEvent,
) -> (
    HeaderMap,
    Sse<impl Stream<Item = Result<Event, Infallible>>>,
) {
    let fallback_data = fallback.data_json().unwrap_or_default();
    let events = stream::unfold(Some(receiver), move |receiver| {
        let fallback_data = fallback_data.clone();
        async move {
            let mut receiver = receiver?;
            let event = receiver.recv().await?;
            match Event::default().event(event.event_name()).json_data(&event) {
                Ok(encoded) => Some((Ok(encoded), Some(receiver))),
                Err(_) => {
                    tracing::error!("failed to serialize a chat SSE event");
                    Some((
                        Ok(Event::default().event("error").data(fallback_data)),
                        None,
                    ))
                }
            }
        }
    });
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store, no-transform"),
    );
    headers.insert("x-accel-buffering", HeaderValue::from_static("no"));
    (
        headers,
        Sse::new(events).keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive"),
        ),
    )
}
