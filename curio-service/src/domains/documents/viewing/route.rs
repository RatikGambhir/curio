use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get},
};

use super::handler;
use crate::domains::documents::Viewing;

/// Library routes, relative to `/v1/documents`.
pub(crate) fn routes(viewing: Arc<Viewing>) -> Router {
    Router::new()
        .route("/", get(handler::list_documents))
        .route("/{file_id}", delete(handler::delete_document))
        .route("/{file_id}/chunks", get(handler::document_chunks))
        .route("/{file_id}/text", get(handler::document_text))
        .route("/{file_id}/pdf", get(handler::document_pdf))
        .with_state(viewing)
}
