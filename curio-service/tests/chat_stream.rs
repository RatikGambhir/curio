//! `POST /v1/chat/stream` — normalizing the provider's SSE into Curio's own
//! `token` / `done` / `error` events, and keeping provider detail out of them.

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{json_body, start_mock_openai, test_config, text_body};
use curio_service::app_with_config;
use serde_json::json;
use tower::ServiceExt;

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
async fn chat_route_normalizes_openai_streams() {
    let provider_stream = concat!(
        "data: {\"type\":\"response.output_text.delta\",\"delta\":\"Hello\"}\n\n",
        "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"response-1\"}}\n\n"
    );
    let (base_url, mock_task) = start_mock_openai(StatusCode::OK, provider_stream).await;

    let app = app_with_config(test_config(base_url)).await.unwrap();
    let response = app.clone().oneshot(chat_request()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/event-stream"
    );

    let body = text_body(response).await;
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
        .unwrap();
    let history = json_body(history).await;
    assert_eq!(history["messages"][0]["content"], "Hello");
    assert_eq!(history["messages"][1]["content"], "Hello");
    assert_eq!(history["messages"][1]["status"], "completed");
    assert_eq!(history["messages"][1]["responseId"], "response-1");

    mock_task.abort();
}

#[tokio::test]
async fn provider_http_failures_are_sanitized() {
    let sensitive_detail = "upstream detail that must not reach clients";
    let (base_url, mock_task) =
        start_mock_openai(StatusCode::TOO_MANY_REQUESTS, sensitive_detail).await;

    let response = app_with_config(test_config(base_url))
        .await
        .unwrap()
        .oneshot(chat_request())
        .await
        .unwrap();
    let body = text_body(response).await;

    assert!(body.contains("event: error"));
    assert!(body.contains(r#""code":"provider_error""#));
    assert!(!body.contains(sensitive_detail));

    mock_task.abort();
}
