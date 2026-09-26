//! Full-router and PostgreSQL contract tests for Tasks.
#[path = "support/postgres.rs"]
mod postgres;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    response::Response,
};
use curio_service::{
    adapters::postgres::client::Database,
    app::config::{DocumentsConfig, ServiceConfig},
    app_with_database,
};
use http_body_util::BodyExt;
use postgres::PostgresFixture;
use serde_json::{Value, json};
use tower::ServiceExt;

fn router(database: Database, fixture: &PostgresFixture) -> Router {
    app_with_database(
        ServiceConfig {
            openai_api_key: "test-only".into(),
            openai_model: "test-only".into(),
            openai_base_url: "http://127.0.0.1:1".into(),
            database_url: fixture.database_url().into(),
            database_schema: fixture.schema().into(),
            database_max_connections: 5,
            database_acquire_timeout_seconds: 5,
            cors_allowed_origins: vec!["http://localhost:5173".into()],
            documents: DocumentsConfig::default(),
        },
        database,
    )
}

async fn setup(name: &str) -> Option<(Router, PostgresFixture)> {
    let fixture = PostgresFixture::provision(name).await?;
    let app = router(fixture.database().clone(), &fixture);
    for owner in ["user-a", "user-b"] {
        let response = request(
            &app,
            "POST",
            "/v1/users",
            Some(owner),
            Some(json!({
                "id":owner, "name":owner, "email":format!("{owner}@example.com")
            })),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
    }
    Some((app, fixture))
}

async fn request(
    app: &Router,
    method: &str,
    path: &str,
    owner: Option<&str>,
    body: Option<Value>,
) -> Response {
    raw_request(
        app,
        method,
        path,
        owner,
        body.map(|value| value.to_string()),
        true,
    )
    .await
}

async fn raw_request(
    app: &Router,
    method: &str,
    path: &str,
    owner: Option<&str>,
    body: Option<String>,
    content_type: bool,
) -> Response {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(owner) = owner {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {owner}"));
    }
    if content_type {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    app.clone()
        .oneshot(
            builder
                .body(body.map(Body::from).unwrap_or_else(Body::empty))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn json_body(response: Response) -> Value {
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

async fn api(
    app: &Router,
    method: &str,
    path: &str,
    owner: &str,
    body: Option<Value>,
    expected: u16,
) -> Value {
    let response = request(app, method, path, Some(owner), body).await;
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("JSON response")
    };
    assert_eq!(status.as_u16(), expected, "{method} {path}: {value}");
    value
}
async fn create(app: &Router, owner: &str, input: Value) -> Value {
    api(app, "POST", "/v1/tasks", owner, Some(input), 201).await
}
async fn space(app: &Router, owner: &str, name: &str) -> Value {
    api(
        app,
        "POST",
        "/v1/spaces",
        owner,
        Some(json!({"name":name})),
        201,
    )
    .await
}
async fn tag(app: &Router, owner: &str, name: &str) -> Value {
    api(
        app,
        "POST",
        "/v1/task-tags",
        owner,
        Some(json!({"name":name})),
        201,
    )
    .await
}
fn task_path(task: &Value) -> String {
    format!("/v1/tasks/{}", task["id"].as_str().unwrap())
}
async fn get(app: &Router, owner: &str, task: &Value) -> Value {
    api(app, "GET", &task_path(task), owner, None, 200).await
}
async fn patch(app: &Router, owner: &str, task: &Value, mut body: Value) -> Value {
    body["expectedVersion"] = task["version"].clone();
    api(app, "PATCH", &task_path(task), owner, Some(body), 200).await
}
async fn list(app: &Router, owner: &str, query: &str) -> Value {
    api(app, "GET", &format!("/v1/tasks{query}"), owner, None, 200).await
}
fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[path = "tasks/access.rs"]
mod access;
#[path = "tasks/core.rs"]
mod core;
#[path = "tasks/database.rs"]
mod database;
#[path = "tasks/filters.rs"]
mod filters;
#[path = "tasks/hierarchy.rs"]
mod hierarchy;
#[path = "tasks/subresources.rs"]
mod subresources;
