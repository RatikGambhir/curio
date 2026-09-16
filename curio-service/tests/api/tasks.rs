use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    response::Response,
};
use curio_service::{app_with_database, config::ServiceConfig, database::Database};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::postgres::PostgresFixture;

async fn task_app(test_name: &str) -> Option<(Router, PostgresFixture)> {
    let postgres = PostgresFixture::provision(test_name).await?;
    let app = configured_app(postgres.database().clone(), &postgres);

    for id in ["user-a", "user-b", "user-c"] {
        let response = app
            .clone()
            .oneshot(authenticated_json(
                Request::post("/v1/users"),
                json!({
                    "id": id,
                    "name": id,
                    "email": format!("{id}@example.com")
                }),
                id,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    Some((app, postgres))
}

fn configured_app(database: Database, postgres: &PostgresFixture) -> Router {
    app_with_database(
        ServiceConfig {
            openai_api_key: "local-test-value".to_owned(),
            openai_model: "configured-test-value".to_owned(),
            openai_base_url: "http://127.0.0.1:1".to_owned(),
            database_url: postgres.database_url().to_owned(),
            database_schema: postgres.schema().to_owned(),
            database_max_connections: 5,
            database_acquire_timeout_seconds: 5,
            cors_allowed_origins: vec!["http://localhost:5173".to_owned()],
        },
        database,
    )
}

fn authenticated_json(
    builder: axum::http::request::Builder,
    body: Value,
    bearer_token: &str,
) -> Request<Body> {
    builder
        .header(header::AUTHORIZATION, format!("Bearer {bearer_token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn create_request_as(bearer_token: &str, task: Value) -> Request<Body> {
    authenticated_json(Request::post("/v1/tasks"), task, bearer_token)
}

fn list_request_as(bearer_token: &str, user_id: &str) -> Request<Body> {
    Request::get(format!("/v1/tasks?userId={user_id}"))
        .header(header::AUTHORIZATION, format!("Bearer {bearer_token}"))
        .body(Body::empty())
        .unwrap()
}

async fn json_body(response: Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn task(id: &str, user_id: &str, name: &str) -> Value {
    json!({
        "id": id,
        "userId": user_id,
        "name": name,
        "setAt": "2026-09-01T12:00:00.000Z"
    })
}

fn assert_millisecond_utc(timestamp: &Value) {
    let timestamp = timestamp.as_str().expect("timestamp must be a string");
    chrono::DateTime::parse_from_rfc3339(timestamp).expect("timestamp must be RFC 3339");
    assert_eq!(timestamp.len(), 24);
    assert_eq!(&timestamp[19..20], ".");
    assert_eq!(&timestamp[23..], "Z");
}

#[tokio::test]
async fn create_and_list_enforce_authentication_and_ownership() {
    let Some((app, postgres)) = task_app("task_authentication_and_ownership").await else {
        return;
    };

    let unauthenticated_create = app
        .clone()
        .oneshot(
            Request::post("/v1/tasks")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(task("task-1", "user-a", "Task").to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let unauthenticated_list = app
        .clone()
        .oneshot(
            Request::get("/v1/tasks?userId=user-a")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let cross_user_create = app
        .clone()
        .oneshot(create_request_as(
            "user-a",
            task("task-2", "user-b", "Task"),
        ))
        .await
        .unwrap();
    let cross_user_list = app
        .oneshot(list_request_as("user-a", "user-b"))
        .await
        .unwrap();

    assert_eq!(unauthenticated_create.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(unauthenticated_list.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(cross_user_create.status(), StatusCode::FORBIDDEN);
    assert_eq!(cross_user_list.status(), StatusCode::FORBIDDEN);
    assert!(json_body(cross_user_create).await["error"].is_string());
    assert!(json_body(cross_user_list).await["error"].is_string());

    postgres.cleanup().await;
}

#[tokio::test]
async fn create_normalizes_defaults_and_round_trips_camel_case() {
    let Some((app, postgres)) = task_app("task_create_normalizes_and_round_trips").await else {
        return;
    };

    let response = app
        .clone()
        .oneshot(create_request_as(
            "user-a",
            json!({
                "id": "  task-1  ",
                "userId": " user-a ",
                "name": "  Draft the launch brief  ",
                "description": "   ",
                "status": "   ",
                "priority": " high ",
                "active": false,
                "setAt": "2026-09-01T12:00:00-05:00",
                "dueAt": "2026-09-01T17:00:00Z"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let created = json_body(response).await;

    assert_eq!(created["id"], "task-1");
    assert_eq!(created["userId"], "user-a");
    assert_eq!(created["name"], "Draft the launch brief");
    assert!(created["description"].is_null());
    assert_eq!(created["status"], "scheduled");
    assert_eq!(created["priority"], "high");
    assert_eq!(created["active"], false);
    assert_eq!(created["setAt"], "2026-09-01T17:00:00.000Z");
    assert_eq!(created["dueAt"], "2026-09-01T17:00:00.000Z");
    assert_millisecond_utc(&created["createdAt"]);
    assert_millisecond_utc(&created["updatedAt"]);

    let generated = app
        .clone()
        .oneshot(create_request_as(
            "user-a",
            json!({ "id": "   ", "userId": "user-a", "name": "Generated id" }),
        ))
        .await
        .unwrap();
    assert_eq!(generated.status(), StatusCode::CREATED);
    let generated = json_body(generated).await;
    assert!(uuid::Uuid::parse_str(generated["id"].as_str().unwrap()).is_ok());
    assert!(generated["active"].as_bool().unwrap());
    assert_millisecond_utc(&generated["setAt"]);
    assert!(generated["dueAt"].is_null());

    let listed = app
        .oneshot(list_request_as("user-a", "user-a"))
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    assert_eq!(
        json_body(listed).await["tasks"].as_array().unwrap().len(),
        2
    );

    postgres.cleanup().await;
}

#[tokio::test]
async fn lists_only_the_owner_in_deterministic_order_and_preserves_hostile_text() {
    let Some((app, postgres)) = task_app("task_list_scope_order_and_injection").await else {
        return;
    };
    let hostile = "literal'); DELETE FROM users; --";

    for value in [
        json!({
            "id": "task-b",
            "userId": "user-a",
            "name": hostile,
            "description": hostile,
            "setAt": "2026-09-01T12:00:00Z"
        }),
        task("task-a", "user-a", "First by id"),
        task("task-other", "user-b", "Other owner"),
    ] {
        let bearer = value["userId"].as_str().unwrap().to_owned();
        let response = app
            .clone()
            .oneshot(create_request_as(&bearer, value))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    sqlx::query("UPDATE tasks SET created_at = '2026-09-01T12:00:00Z' WHERE user_id = $1")
        .bind("user-a")
        .execute(postgres.database().pool())
        .await
        .unwrap();

    let mine = json_body(
        app.clone()
            .oneshot(list_request_as("user-a", "user-a"))
            .await
            .unwrap(),
    )
    .await;
    let empty = json_body(
        app.oneshot(list_request_as("user-c", "user-c"))
            .await
            .unwrap(),
    )
    .await;

    assert_eq!(mine["tasks"].as_array().unwrap().len(), 2);
    assert_eq!(mine["tasks"][0]["id"], "task-a");
    assert_eq!(mine["tasks"][1]["id"], "task-b");
    assert_eq!(mine["tasks"][1]["name"], hostile);
    assert_eq!(mine["tasks"][1]["description"], hostile);
    assert!(empty["tasks"].as_array().unwrap().is_empty());

    let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(postgres.database().pool())
        .await
        .unwrap();
    assert_eq!(users, 3);

    postgres.cleanup().await;
}

#[tokio::test]
async fn validation_conflict_and_unknown_user_errors_are_structured() {
    let Some((app, postgres)) = task_app("task_validation_conflict_unknown_user").await else {
        return;
    };
    let created = app
        .clone()
        .oneshot(create_request_as(
            "user-a",
            task("duplicate-task", "user-a", "Task"),
        ))
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);

    let cases = [
        (
            task("duplicate-task", "user-a", "Task"),
            StatusCode::CONFLICT,
        ),
        (
            json!({ "userId": "user-a", "name": " " }),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (json!({ "name": "Task" }), StatusCode::UNPROCESSABLE_ENTITY),
        (
            json!({ "userId": "user-a", "name": "Task", "status": "unknown" }),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            json!({ "userId": "user-a", "name": "Task", "priority": "urgent" }),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            json!({ "userId": "user-a", "name": "Task", "setAt": "tomorrow" }),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            json!({ "userId": "user-a", "name": "Task", "dueAt": "tomorrow" }),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            json!({
                "userId": "user-a",
                "name": "Task",
                "setAt": "2026-09-02T00:00:00Z",
                "dueAt": "2026-09-01T00:00:00Z"
            }),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
    ];

    for (body, expected_status) in cases {
        let response = app
            .clone()
            .oneshot(create_request_as("user-a", body))
            .await
            .unwrap();
        assert_eq!(response.status(), expected_status);
        assert!(json_body(response).await["error"].is_string());
    }

    let unknown_user = app
        .clone()
        .oneshot(create_request_as(
            "missing-user",
            task("missing-user-task", "missing-user", "Task"),
        ))
        .await
        .unwrap();
    assert_eq!(unknown_user.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        json_body(unknown_user).await["error"],
        "No profile exists for that user yet."
    );

    let blank_list = app.oneshot(list_request_as("user-a", "")).await.unwrap();
    assert_eq!(blank_list.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(json_body(blank_list).await["error"].is_string());

    postgres.cleanup().await;
}

#[tokio::test]
async fn created_task_survives_closing_and_reopening_the_pool() {
    let Some((app, postgres)) = task_app("task_survives_pool_reconnect").await else {
        return;
    };
    let response = app
        .oneshot(create_request_as(
            "user-a",
            task("durable-task", "user-a", "Durable"),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    postgres.database().close().await;
    let reopened_database = postgres.reconnect().await;
    let reopened_app = configured_app(reopened_database.clone(), &postgres);
    let response = reopened_app
        .oneshot(list_request_as("user-a", "user-a"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["tasks"].as_array().unwrap().len(), 1);
    assert_eq!(body["tasks"][0]["id"], "durable-task");

    reopened_database.close().await;
    postgres.cleanup().await;
}
