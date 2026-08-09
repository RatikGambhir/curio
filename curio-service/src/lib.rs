pub mod chat;
pub mod config;
mod user;

use axum::{
    Json, Router,
    extract::Request,
    http::{StatusCode, header},
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct CurrentUser;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    status: &'static str,
}

pub fn app() -> Router {
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

async fn auth(mut request: Request, next: Next) -> Result<Response, StatusCode> {
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
        .filter(|token| !token.trim().is_empty())
        .map(|_| CurrentUser)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
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
                    .header(header::AUTHORIZATION, "Bearer development-token")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"message":"Hello"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
