use axum::{Router, routing::get};

use super::handler::{health_handler, readiness_handler, root_handler};
use crate::adapters::postgres::client::Database;

/// Liveness routes; they never touch the database.
pub(crate) fn routes() -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
}

/// Database access plus expected migration-row verification.
pub(crate) fn readiness_routes(database: Database) -> Router {
    Router::new()
        .route("/ready", get(readiness_handler))
        .with_state(database)
}
