use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::{
    database::Database,
    user::{handler, repository::UserRepository, service::UserService},
};

pub fn placeholder_routes() -> Router {
    Router::new()
        .route("/conversations", post(handler::create_conversation))
        .route(
            "/conversations/{id}",
            get(handler::get_conversation).post(handler::update_conversation),
        )
}

pub fn api_routes(database: Database) -> Router {
    let service = UserService::new(UserRepository::new(database));

    Router::new()
        .route("/v1/users", post(handler::save_user))
        .route_layer(middleware::from_fn(crate::auth))
        .with_state(service)
}
