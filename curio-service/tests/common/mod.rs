//! Fixtures shared by the integration tests.
//!
//! Cargo compiles this module separately into every test binary, so a helper
//! that only one file uses would warn as dead code in all the others.
#![allow(dead_code)]

pub mod calendar;

use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use curio_service::config::ServiceConfig;
use http_body_util::BodyExt;
use serde_json::Value;
use tokio::task::JoinHandle;

/// Any non-empty bearer token authorizes today. The token does not say *who*,
/// which is why every user-scoped request also carries an explicit id.
pub const TEST_TOKEN: &str = "Bearer development-token";

pub fn test_config(openai_base_url: String) -> ServiceConfig {
    ServiceConfig {
        openai_api_key: "local-test-value".to_owned(),
        openai_model: "configured-test-value".to_owned(),
        openai_base_url,
        database_url: "sqlite::memory:".to_owned(),
        cors_allowed_origins: vec!["http://localhost:5173".to_owned()],
    }
}

/// A config pointed at a port nothing listens on, for the tests that never
/// reach the provider.
pub fn offline_config() -> ServiceConfig {
    test_config("http://127.0.0.1:1".to_owned())
}

/// An authenticated JSON request, the shape almost every test needs.
pub fn authenticated(request: axum::http::request::Builder, body: Value) -> Request<Body> {
    request
        .header(header::AUTHORIZATION, TEST_TOKEN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

pub async fn json_body(response: Response) -> Value {
    let body = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&body).unwrap()
}

pub async fn text_body(response: Response) -> String {
    let body = response.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(body.to_vec()).unwrap()
}

#[derive(Clone)]
struct MockOpenAiResponse {
    status: StatusCode,
    body: String,
}

async fn mock_responses(
    State(response): State<MockOpenAiResponse>,
    headers: HeaderMap,
    Json(request): Json<Value>,
) -> impl IntoResponse {
    assert!(
        headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("Bearer "))
    );
    assert_eq!(request["stream"], true);

    (
        response.status,
        [(header::CONTENT_TYPE, "text/event-stream")],
        response.body,
    )
}

/// Stands in for the OpenAI Responses API on a loopback port, so the chat tests
/// exercise the real HTTP path without reaching the network.
pub async fn start_mock_openai(
    status: StatusCode,
    body: impl Into<String>,
) -> (String, JoinHandle<()>) {
    let mock = Router::new()
        .route("/v1/responses", post(mock_responses))
        .with_state(MockOpenAiResponse {
            status,
            body: body.into(),
        });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(listener, mock).await.unwrap();
    });

    (format!("http://{address}"), task)
}
