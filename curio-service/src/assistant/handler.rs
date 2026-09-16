use std::convert::Infallible;

use axum::{
    Json,
    extract::{Path, State},
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use futures_util::StreamExt;
use tracing::{Instrument, info_span};

use crate::{
    assistant::{
        model::{ChatStreamRequest, ConversationMessagesResponse, ConversationsResponse},
        service::AssistantService,
    },
    diagnostics,
};

pub async fn stream_chat(
    State(service): State<AssistantService>,
    Json(request): Json<ChatStreamRequest>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let events = service
        .stream_chat(request)
        .await
        .map(|event| Ok(Event::default().event(event.name).data(event.data)));

    Sse::new(events).keep_alive(KeepAlive::default())
}

pub async fn list_conversations(
    State(service): State<AssistantService>,
) -> Result<Json<ConversationsResponse>, axum::http::StatusCode> {
    match service.list_conversations().await {
        Ok(conversations) => Ok(Json(ConversationsResponse { conversations })),
        Err(storage_error) => {
            diagnostics::log_database_error("chat_list_conversations", &storage_error);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn conversation_messages(
    State(service): State<AssistantService>,
    Path(conversation_id): Path<String>,
) -> Result<Json<ConversationMessagesResponse>, axum::http::StatusCode> {
    let history_span = info_span!(
        "conversation_messages",
        conversation_id = ?conversation_id,
    );
    let result = service
        .conversation_messages(&conversation_id)
        .instrument(history_span.clone())
        .await;

    match result {
        Ok(messages) => Ok(Json(ConversationMessagesResponse {
            conversation_id,
            messages,
        })),
        Err(storage_error) => {
            history_span.in_scope(|| {
                diagnostics::log_database_error("chat_conversation_messages", &storage_error);
            });
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
