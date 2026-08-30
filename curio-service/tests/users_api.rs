//! `POST /v1/users` — the bearer-protected profile upsert.

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{TEST_TOKEN, authenticated, json_body, offline_config};
use curio_service::app_with_config;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn saving_a_user_requires_authentication() {
    let app = app_with_config(offline_config()).await.unwrap();

    let response = app
        .oneshot(
            Request::post("/v1/users")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "id": "user-1", "name": "Curio", "email": "curio@example.com" })
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn saving_a_user_upserts_the_profile() {
    let app = app_with_config(offline_config()).await.unwrap();

    let save = |name: &str| {
        authenticated(
            Request::post("/v1/users"),
            json!({
                "id": "user-1",
                "name": name,
                "email": "curio@example.com",
                "avatarUrl": "https://example.com/avatar.png"
            }),
        )
    };

    let created = app.clone().oneshot(save("Curio")).await.unwrap();
    assert_eq!(created.status(), StatusCode::OK);
    let created = json_body(created).await;
    assert_eq!(created["id"], "user-1");
    assert_eq!(created["name"], "Curio");
    assert_eq!(created["email"], "curio@example.com");
    assert_eq!(created["avatarUrl"], "https://example.com/avatar.png");

    let updated = app.oneshot(save("Curio Renamed")).await.unwrap();
    assert_eq!(updated.status(), StatusCode::OK);
    assert_eq!(json_body(updated).await["name"], "Curio Renamed");
}

#[tokio::test]
async fn saving_a_user_rejects_blank_profiles() {
    let app = app_with_config(offline_config()).await.unwrap();

    let response = app
        .oneshot(
            Request::post("/v1/users")
                .header(header::AUTHORIZATION, TEST_TOKEN)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "id": "user-1", "name": "  ", "email": "" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
