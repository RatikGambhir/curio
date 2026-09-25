use std::sync::Arc;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

use super::handler;
use crate::domains::documents::{Ingestion, Jobs, model::MAX_TOTAL_REQUEST_FILE_BYTES};

/// Multipart framing headroom on top of the file-byte limit.
const MULTIPART_OVERHEAD_BYTES: usize = 1024 * 1024;
const UPLOAD_BODY_LIMIT: usize = MAX_TOTAL_REQUEST_FILE_BYTES + MULTIPART_OVERHEAD_BYTES;

/// Upload and job routes, relative to `/v1/documents`.
pub(crate) fn routes(ingestion: Arc<Ingestion>, jobs: Arc<Jobs>) -> Router {
    let ingestion_routes = Router::new()
        .route(
            "/process",
            post(handler::process_documents).layer(DefaultBodyLimit::max(UPLOAD_BODY_LIMIT)),
        )
        .with_state(ingestion);
    let job_routes = Router::new()
        .route(
            "/jobs",
            post(handler::start_job).layer(DefaultBodyLimit::max(UPLOAD_BODY_LIMIT)),
        )
        .route("/jobs/{job_id}/events", get(handler::job_events))
        .with_state(jobs);

    ingestion_routes.merge(job_routes)
}
