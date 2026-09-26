use super::{handler, service::TaskService};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, patch, post},
};
use std::sync::Arc;

pub(crate) fn routes(service: Arc<TaskService>) -> Router {
    Router::new()
        .route("/v1/tasks", post(handler::create).get(handler::list))
        .route(
            "/v1/tasks/{id}",
            get(handler::get)
                .patch(handler::patch)
                .delete(handler::delete),
        )
        .route(
            "/v1/task-tags",
            post(handler::create_tag).get(handler::tags),
        )
        .route(
            "/v1/task-tags/{id}",
            patch(handler::rename_tag).delete(handler::delete_tag),
        )
        .route(
            "/v1/tasks/{id}/comments",
            post(handler::create_comment).get(handler::comments),
        )
        .route(
            "/v1/tasks/{id}/comments/{commentId}",
            patch(handler::edit_comment).delete(handler::delete_comment),
        )
        .route("/v1/tasks/{id}/links", post(handler::create_link))
        .route(
            "/v1/tasks/{id}/links/{linkId}",
            patch(handler::edit_link).delete(handler::delete_link),
        )
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .with_state(service)
}
