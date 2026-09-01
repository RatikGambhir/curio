use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode, header},
    response::IntoResponse,
    routing::post,
};
use curio_service::{app, app_with_database, config::ServiceConfig};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tokio::task::JoinHandle;
use tower::ServiceExt;

use crate::postgres::PostgresFixture;

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

async fn start_mock_openai(
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

fn test_config(openai_base_url: String, postgres: &PostgresFixture) -> ServiceConfig {
    ServiceConfig {
        openai_api_key: "local-test-value".to_owned(),
        openai_model: "configured-test-value".to_owned(),
        openai_base_url,
        database_url: postgres.database_url().to_owned(),
        database_schema: postgres.schema().to_owned(),
        database_max_connections: 5,
        database_acquire_timeout_seconds: 5,
        cors_allowed_origins: vec!["http://localhost:5173".to_owned()],
    }
}

async fn test_app(test_name: &str, openai_base_url: String) -> Option<(Router, PostgresFixture)> {
    let postgres = PostgresFixture::provision(test_name).await?;
    let app = configured_app(openai_base_url, &postgres);
    Some((app, postgres))
}

fn configured_app(openai_base_url: String, postgres: &PostgresFixture) -> Router {
    app_with_database(
        test_config(openai_base_url, postgres),
        postgres.database().clone(),
    )
}

fn assert_millisecond_utc(timestamp: &Value) {
    let timestamp = timestamp.as_str().expect("timestamp must be a string");
    chrono::DateTime::parse_from_rfc3339(timestamp).expect("timestamp must be RFC 3339");
    assert_eq!(
        timestamp.len(),
        24,
        "timestamp must have millisecond precision"
    );
    assert_eq!(&timestamp[19..20], ".");
    assert_eq!(&timestamp[23..], "Z");
}

fn chat_request() -> Request<Body> {
    Request::post("/v1/chat/stream")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "conversationId": "conversation-1",
                "userMessageId": "user-1",
                "assistantMessageId": "assistant-1",
                "prompt": "Hello"
            })
            .to_string(),
        ))
        .unwrap()
}

#[tokio::test]
async fn health_is_public() {
    let response = app()
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn placeholder_user_routes_require_authentication() {
    let response = app()
        .oneshot(
            Request::get("/user/conversations/conversation-1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn authenticated_routes_deserialize_json_requests() {
    let response = app()
        .oneshot(
            Request::post("/user/conversations")
                .header(header::AUTHORIZATION, "Bearer development-token")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"message":"Hello"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn saving_a_user_requires_authentication() {
    let Some((app, postgres)) = test_app(
        "saving_a_user_requires_authentication",
        "http://127.0.0.1:1".to_owned(),
    )
    .await
    else {
        return;
    };

    let response = app
        .oneshot(
            Request::post("/v1/users")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"id":"user-1","name":"Curio","email":"curio@example.com"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    postgres.cleanup().await;
}

#[tokio::test]
async fn saving_a_user_upserts_the_profile() {
    let Some((app, postgres)) = test_app(
        "saving_a_user_upserts_the_profile",
        "http://127.0.0.1:1".to_owned(),
    )
    .await
    else {
        return;
    };

    let save = |id: &str, name: &str| {
        Request::post("/v1/users")
            .header(header::AUTHORIZATION, "Bearer development-token")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "id": id,
                    "name": name,
                    "email": "curio@example.com",
                    "avatarUrl": "https://example.com/avatar.png"
                })
                .to_string(),
            ))
            .unwrap()
    };

    let created = app.clone().oneshot(save("user-1", "Curio")).await.unwrap();
    assert_eq!(created.status(), StatusCode::OK);
    let created = created.into_body().collect().await.unwrap().to_bytes();
    let created: Value = serde_json::from_slice(&created).unwrap();
    assert_eq!(created["id"], "user-1");
    assert_eq!(created["name"], "Curio");
    assert_eq!(created["email"], "curio@example.com");
    assert_eq!(created["avatarUrl"], "https://example.com/avatar.png");
    assert_millisecond_utc(&created["createdAt"]);
    assert_millisecond_utc(&created["updatedAt"]);

    let updated = app
        .clone()
        .oneshot(save("user-1", "Curio Renamed"))
        .await
        .unwrap();
    assert_eq!(updated.status(), StatusCode::OK);
    let updated = updated.into_body().collect().await.unwrap().to_bytes();
    let updated: Value = serde_json::from_slice(&updated).unwrap();
    assert_eq!(updated["name"], "Curio Renamed");

    let duplicate_email = app.oneshot(save("user-2", "Another Curio")).await.unwrap();
    assert_eq!(duplicate_email.status(), StatusCode::CONFLICT);

    postgres.cleanup().await;
}

#[tokio::test]
async fn saving_a_user_rejects_blank_profiles() {
    let Some((app, postgres)) = test_app(
        "saving_a_user_rejects_blank_profiles",
        "http://127.0.0.1:1".to_owned(),
    )
    .await
    else {
        return;
    };

    let response = app
        .oneshot(
            Request::post("/v1/users")
                .header(header::AUTHORIZATION, "Bearer development-token")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"id":"user-1","name":"  ","email":""}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    postgres.cleanup().await;
}

#[tokio::test]
async fn chat_route_normalizes_openai_streams() {
    let Some(postgres) = PostgresFixture::provision("chat_route_normalizes_openai_streams").await
    else {
        return;
    };
    let provider_stream = concat!(
        "data: {\"type\":\"response.output_text.delta\",\"delta\":\"Hello\"}\n\n",
        "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"response-1\"}}\n\n"
    );
    let (base_url, mock_task) = start_mock_openai(StatusCode::OK, provider_stream).await;
    let app = configured_app(base_url, &postgres);
    let response = app.clone().oneshot(chat_request()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/event-stream"
    );

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert!(body.contains("event: token"));
    assert!(body.contains(
        r#"data: {"conversationId":"conversation-1","messageId":"assistant-1","token":"Hello"}"#
    ));
    assert!(body.contains("event: done"));
    assert!(body.contains(r#""responseId":"response-1""#));

    let history = app
        .oneshot(
            Request::get("/v1/conversations/conversation-1/messages")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let history: Value = serde_json::from_slice(&history).unwrap();
    assert_eq!(history["messages"][0]["content"], "Hello");
    assert_eq!(history["messages"][1]["content"], "Hello");
    assert_eq!(history["messages"][1]["status"], "completed");
    assert_eq!(history["messages"][1]["responseId"], "response-1");
    assert_millisecond_utc(&history["messages"][0]["createdAt"]);
    assert_millisecond_utc(&history["messages"][1]["updatedAt"]);

    mock_task.abort();
    postgres.cleanup().await;
}

#[tokio::test]
async fn provider_http_failures_propagate_detail_without_credentials() {
    let Some(postgres) =
        PostgresFixture::provision("provider_http_failures_propagate_detail_without_credentials")
            .await
    else {
        return;
    };
    // A real 429/401 body quotes the rejected credential back at us. The
    // diagnostic half must reach the client; the credential must not.
    let secret = "sk-proj-MUSTNOTLEAK123";
    let provider_body = format!(
        r#"{{"error":{{"message":"Rate limit reached. Incorrect API key provided: {secret}.","type":"requests","code":"rate_limit_exceeded"}}}}"#
    );
    let (base_url, mock_task) =
        start_mock_openai(StatusCode::TOO_MANY_REQUESTS, provider_body).await;
    let app = configured_app(base_url, &postgres);
    let response = app.oneshot(chat_request()).await.unwrap();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8(body.to_vec()).unwrap();

    assert!(body.contains("event: error"));
    assert!(body.contains(r#""code":"provider_rate_limited""#));
    assert!(body.contains("Rate limit reached."));
    assert!(body.contains("rate_limit_exceeded"));
    assert!(body.contains("HTTP 429"));
    assert!(
        !body.contains(secret),
        "the API key reached the client: {body}"
    );
    assert!(body.contains("sk-***"));

    mock_task.abort();
    postgres.cleanup().await;
}

#[tokio::test]
async fn configured_web_origin_receives_cors_headers() {
    let Some((app, postgres)) = test_app(
        "configured_web_origin_receives_cors_headers",
        "http://127.0.0.1:1".to_owned(),
    )
    .await
    else {
        return;
    };
    let response = app
        .oneshot(
            Request::options("/v1/chat/stream")
                .header(header::ORIGIN, "http://localhost:5173")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "http://localhost:5173"
    );
    postgres.cleanup().await;
}

#[tokio::test]
async fn readiness_reflects_database_availability() {
    let Some((app, postgres)) = test_app(
        "readiness_reflects_database_availability",
        "http://127.0.0.1:1".to_owned(),
    )
    .await
    else {
        return;
    };

    let ready = app
        .clone()
        .oneshot(Request::get("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(ready.status(), StatusCode::OK);

    postgres.database().close().await;
    let unavailable = app
        .oneshot(Request::get("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(unavailable.status(), StatusCode::SERVICE_UNAVAILABLE);

    postgres.cleanup().await;
}
