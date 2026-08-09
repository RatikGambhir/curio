mod commands;
mod queries;

use axum::{
    Router,
    routing::{get, post},
};

pub fn routes() -> Router {
    Router::new()
        .route("/conversations", post(commands::create_conversation))
        .route(
            "/conversations/{id}",
            get(queries::get_conversation).post(commands::update_conversation),
        )
}
