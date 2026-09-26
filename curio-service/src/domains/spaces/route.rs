use super::{Service, handler};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{delete, post},
};
use std::sync::Arc;

pub(crate) fn routes(service: Arc<Service>) -> Router {
    Router::new()
        .route(
            "/v1/spaces",
            post(handler::create)
                .layer(DefaultBodyLimit::max(16 * 1024))
                .get(handler::list),
        )
        .route("/v1/spaces/{id}", delete(handler::delete))
        .with_state(service)
}
