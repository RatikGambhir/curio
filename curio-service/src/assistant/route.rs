use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    assistant::{handler, repository::AssistantRepository, service::AssistantService},
    database::Database,
};

pub fn routes(
    openai_api_key: String,
    openai_model: String,
    openai_base_url: String,
    database: Database,
) -> Router {
    let service = AssistantService::new(
        openai_api_key,
        openai_model,
        openai_base_url,
        AssistantRepository::new(database),
    );

    Router::new()
        .route("/v1/chat/stream", post(handler::stream_chat))
        .route("/v1/conversations", get(handler::list_conversations))
        .route(
            "/v1/conversations/{id}/messages",
            get(handler::conversation_messages),
        )
        .with_state(service)
}
