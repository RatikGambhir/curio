mod calendar;
pub mod chat;
pub mod config;
pub mod database;
pub mod diagnostics;
pub mod documents;
pub mod query;
mod serialization;
mod user;

#[cfg(test)]
extern crate self as curio_service;
#[cfg(test)]
#[path = "../tests/support/postgres.rs"]
pub(crate) mod postgres_test_support;

use std::time::Duration;

use axum::{
    Json, Router,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use http::HeaderValue;
use serde::Serialize;
use tower_http::cors::CorsLayer;

use crate::{
    config::ServiceConfig,
    database::{Database, DatabaseOptions},
};

#[derive(Clone, Debug)]
pub struct CurrentUser {
    id: String,
}

impl CurrentUser {
    pub(crate) fn owns(&self, user_id: &str) -> bool {
        self.id == user_id.trim()
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    status: &'static str,
}

pub fn app() -> Router {
    base_router()
}

pub async fn app_with_config(config: ServiceConfig) -> Result<Router, sqlx::Error> {
    let database = Database::connect(&DatabaseOptions {
        url: config.database_url.clone(),
        schema: config.database_schema.clone(),
        max_connections: config.database_max_connections,
        acquire_timeout: Duration::from_secs(config.database_acquire_timeout_seconds),
        application_name: "curio-service".to_owned(),
    })
    .await
    .inspect_err(|error| diagnostics::log_database_error("service_database_connect", error))?;
    database
        .verify_migrations()
        .await
        .inspect_err(|error| diagnostics::log_database_error("service_migration_verify", error))?;

    Ok(app_with_database(config, database))
}

pub fn app_with_database(config: ServiceConfig, database: Database) -> Router {
    let allowed_origins = config
        .cors_allowed_origins
        .iter()
        .map(|origin| HeaderValue::from_str(origin).expect("invalid configured CORS origin"))
        .collect::<Vec<_>>();
    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::PATCH,
            http::Method::DELETE,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);
    let calendar_routes = calendar::router(database.clone());
    let user_routes = user::router(database.clone());
    let document_routes = documents::router(
        database.clone(),
        config.openai_api_key.clone(),
        config.openai_base_url.clone(),
        &config.documents,
    );
    let chat_routes = chat::router(
        database.clone(),
        config.openai_api_key,
        config.openai_model,
        config.openai_base_url,
    );

    let readiness_route = Router::new()
        .route("/ready", get(readiness))
        .with_state(database);

    base_router()
        .merge(readiness_route)
        .merge(chat_routes)
        .merge(calendar_routes)
        .merge(user_routes)
        .merge(document_routes)
        .layer(cors)
}

fn base_router() -> Router {
    let protected_routes = Router::new()
        .nest("/user", user::legacy_router())
        .route_layer(middleware::from_fn(auth));

    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .merge(protected_routes)
}

async fn root() -> &'static str {
    "Hello from curio-service!"
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn readiness(State(database): State<Database>) -> (StatusCode, Json<HealthResponse>) {
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

pub(crate) async fn auth(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let current_user = authorize_current_user(auth_header)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    request.extensions_mut().insert(current_user);

    Ok(next.run(request).await)
}

async fn authorize_current_user(auth_header: &str) -> Option<CurrentUser> {
    auth_header
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty() && !token.as_bytes().iter().any(u8::is_ascii_whitespace))
        .map(|token| CurrentUser {
            id: token.to_owned(),
        })
}
