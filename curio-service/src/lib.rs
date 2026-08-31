mod calendar;
pub mod chat;
pub mod config;
pub mod database;
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
    routing::{get, post},
};
use http::HeaderValue;
use serde::Serialize;
use tower_http::cors::CorsLayer;

use crate::{
    chat::{ChatState, repository::ChatRepository},
    config::ServiceConfig,
    database::{Database, DatabaseOptions},
    user::repository::UserRepository,
};

#[derive(Clone, Debug)]
pub struct CurrentUser {
    id: String,
}

impl CurrentUser {
    pub(crate) fn owns(&self, user_id: &str) -> bool {
        self.id == user_id.trim()
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
    .await?;
    database.verify_migrations().await?;

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
    let calendar_api_routes = calendar::api_routes(database.clone());
    let user_api_routes = user::api_routes(UserRepository::new(database.clone()));
    let chat_state = ChatState::new(
        config.openai_api_key,
        config.openai_model,
        config.openai_base_url,
        ChatRepository::new(database.clone()),
    );

    let chat_routes = Router::new()
        .route("/v1/chat/stream", post(chat::stream_chat))
        .route("/v1/conversations", get(chat::list_conversations))
        .route(
            "/v1/conversations/{id}/messages",
            get(chat::conversation_messages),
        )
        .with_state(chat_state);

    let readiness_route = Router::new()
        .route("/ready", get(readiness))
        .with_state(database);

    base_router()
        .merge(readiness_route)
        .merge(chat_routes)
        .merge(calendar_api_routes)
        .merge(user_api_routes)
        .layer(cors)
}

fn base_router() -> Router {
    let protected_routes = Router::new()
        .nest("/user", user::routes())
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
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "unavailable",
            }),
        ),
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
