use std::sync::Arc;

use axum::{Router, routing::post};

use super::handler;
use crate::domains::documents::Search;

/// Retrieval routes, relative to `/v1/documents`.
pub(crate) fn routes(search: Arc<Search>) -> Router {
    Router::new()
        .route("/search/vector", post(handler::vector_search))
        .route("/search/keyword", post(handler::keyword_search))
        .with_state(search)
}
