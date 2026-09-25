use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use super::{handler, service::ChatService};

pub(crate) fn routes(chat: Arc<ChatService>) -> Router {
    Router::new()
        .route("/v1/chat/stream", post(handler::stream_chat_handler))
        .route(
            "/v1/conversations",
            get(handler::list_conversations_handler),
        )
        .route(
            "/v1/conversations/{id}/messages",
            get(handler::conversation_messages_handler),
        )
        .with_state(handler::ChatHttpState { chat })
}
