mod commands;
mod queries;

use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::database::Database;

pub fn routes() -> Router {
    Router::new()
        .route("/conversations", post(commands::create_conversation))
        .route(
            "/conversations/{id}",
            get(queries::get_conversation).post(commands::update_conversation),
        )
}

/// Authenticated user API routes backed by the database.
pub fn api_routes(database: Database) -> Router {
    Router::new()
        .route("/v1/users", post(commands::save_user))
        .route_layer(middleware::from_fn(crate::auth))
        .with_state(database)
}
