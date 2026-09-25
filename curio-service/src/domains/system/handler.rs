use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::{adapters::postgres::client::Database, shared::diagnostics};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HealthResponse {
    status: &'static str,
}

pub(super) async fn root_handler() -> &'static str {
    "Hello from curio-service!"
}

pub(super) async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub(super) async fn readiness_handler(
    State(database): State<Database>,
) -> (StatusCode, Json<HealthResponse>) {
    match database.readiness().await {
        Ok(()) => (StatusCode::OK, Json(HealthResponse { status: "ok" })),
        Err(error) => {
            diagnostics::log_database_error("readiness_check", &error);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "unavailable",
                }),
            )
        }
    }
}
