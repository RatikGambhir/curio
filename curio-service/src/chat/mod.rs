//! Chat composition root. HTTP, provider and storage adapters meet only here.
pub(crate) mod domain;
mod handlers;
mod openai;
pub mod protocol;
pub(crate) mod repository;
pub(crate) mod service;

use self::{openai::OpenAiClient, repository::PostgresChatRepository, service::ChatService};
use crate::database::Database;
use axum::{
    Router,
    routing::{get, post},
};

type Service = ChatService<PostgresChatRepository, OpenAiClient>;

pub(crate) fn router(
    database: Database,
    api_key: String,
    model: String,
    base_url: String,
) -> Router {
    let service = ChatService::new(
        PostgresChatRepository::new(database),
        OpenAiClient::new(api_key, model, base_url),
    );
    Router::new()
        .route("/v1/chat/stream", post(handlers::stream_chat))
        .route("/v1/conversations", get(handlers::list_conversations))
        .route(
            "/v1/conversations/{id}/messages",
            get(handlers::conversation_messages),
        )
        .with_state(service)
}
