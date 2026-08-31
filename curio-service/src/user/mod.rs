mod commands;
mod queries;
pub(crate) mod repository;

use axum::{
    Router, middleware,
    routing::{get, post},
};

use self::repository::UserRepository;

pub fn routes() -> Router {
    Router::new()
        .route("/conversations", post(commands::create_conversation))
        .route(
            "/conversations/{id}",
            get(queries::get_conversation).post(commands::update_conversation),
        )
}

/// Authenticated user API routes backed by the database.
pub fn api_routes(repository: UserRepository) -> Router {
    Router::new()
        .route("/v1/users", post(commands::save_user))
        .route_layer(middleware::from_fn(crate::auth))
        .with_state(repository)
}
