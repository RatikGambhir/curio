use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use super::{Service, handler, legacy};

/// Database-backed user API routes.
pub(crate) fn routes(users: Arc<Service>) -> Router {
    Router::new()
        .route("/v1/users", post(handler::save_user))
        .with_state(users)
}

/// Non-persistent conversation placeholders, mounted under `/user`.
pub(crate) fn legacy_routes() -> Router {
    Router::new()
        .route("/conversations", post(legacy::create_conversation))
        .route(
            "/conversations/{id}",
            get(legacy::get_conversation).post(legacy::update_conversation),
        )
}
