//! User composition root; legacy placeholders remain isolated in `legacy`.
mod domain;
mod handlers;
mod legacy;
mod repository;
mod service;

use axum::{
    Router, middleware,
    routing::{get, post},
};

use self::{repository::PostgresUserRepository, service::UserService};
use crate::database::Database;

type Service = UserService<PostgresUserRepository>;

pub(crate) fn legacy_router() -> Router {
    Router::new()
        .route("/conversations", post(legacy::create_conversation))
        .route(
            "/conversations/{id}",
            get(legacy::get_conversation).post(legacy::update_conversation),
        )
}

/// Authenticated user API routes backed by the database.
pub(crate) fn router(database: Database) -> Router {
    let service = UserService::new(PostgresUserRepository::new(database));
    Router::new()
        .route("/v1/users", post(handlers::save_user))
        .route_layer(middleware::from_fn(crate::auth))
        .with_state(service)
}
