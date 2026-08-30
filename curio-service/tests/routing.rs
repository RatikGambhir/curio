//! What the router exposes and what it protects: which routes are public, which
//! demand a bearer token, and which origins CORS answers.

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{TEST_TOKEN, offline_config};
use curio_service::{app, app_with_config};
use serde_json::json;
use tower::ServiceExt;

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
                .header(header::AUTHORIZATION, TEST_TOKEN)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({ "message": "Hello" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn configured_web_origin_receives_cors_headers() {
    let response = app_with_config(offline_config())
        .await
        .unwrap()
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
}
