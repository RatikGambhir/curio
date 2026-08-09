pub mod chat;
pub mod config;
mod user;

use axum::{
    Json, Router,
    extract::Request,
    http::{StatusCode, header},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
};
use http::HeaderValue;
use serde::Serialize;
use tower_http::cors::CorsLayer;

use crate::{chat::ChatState, config::ServiceConfig};

#[derive(Clone, Debug)]
pub struct CurrentUser;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    status: &'static str,
}

pub fn app() -> Router {
    base_router()
}

pub fn app_with_config(config: ServiceConfig) -> Router {
    let allowed_origins = config
        .cors_allowed_origins
        .iter()
        .map(|origin| HeaderValue::from_str(origin).expect("invalid configured CORS origin"))
        .collect::<Vec<_>>();
    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([http::Method::GET, http::Method::POST])
        .allow_headers([header::CONTENT_TYPE]);
    let chat_state = ChatState::new(
        config.openai_api_key,
        config.openai_model,
        config.openai_base_url,
    );

    let chat_routes = Router::new()
        .route("/v1/chat/stream", post(chat::stream_chat))
        .with_state(chat_state);

    base_router().merge(chat_routes).layer(cors)
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
    use axum::{
        body::Body,
        extract::State,
        http::{HeaderMap, Request},
        response::IntoResponse,
        routing::post,
    };
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use tokio::task::JoinHandle;
    use tower::ServiceExt;

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

    fn test_config(openai_base_url: String) -> ServiceConfig {
        ServiceConfig {
            openai_api_key: "local-test-value".to_owned(),
            openai_model: "configured-test-value".to_owned(),
            openai_base_url,
            database_url: "sqlite::memory:".to_owned(),
            cors_allowed_origins: vec!["http://localhost:5173".to_owned()],
        }
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
    async fn chat_route_normalizes_openai_streams() {
        let provider_stream = concat!(
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"Hello\"}\n\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"response-1\"}}\n\n"
        );
        let (base_url, mock_task) = start_mock_openai(StatusCode::OK, provider_stream).await;

        let response = app_with_config(test_config(base_url))
            .oneshot(chat_request())
            .await
            .unwrap();
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

        mock_task.abort();
    }

    #[tokio::test]
    async fn provider_http_failures_are_sanitized() {
        let sensitive_detail = "upstream detail that must not reach clients";
        let (base_url, mock_task) =
            start_mock_openai(StatusCode::TOO_MANY_REQUESTS, sensitive_detail).await;

        let response = app_with_config(test_config(base_url))
            .oneshot(chat_request())
            .await
            .unwrap();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert!(body.contains("event: error"));
        assert!(body.contains(r#""code":"provider_error""#));
        assert!(!body.contains(sensitive_detail));

        mock_task.abort();
    }

    #[tokio::test]
    async fn configured_web_origin_receives_cors_headers() {
        let response = app_with_config(test_config("http://127.0.0.1:1".to_owned()))
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
}
