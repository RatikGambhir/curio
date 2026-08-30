mod calendar;
pub mod chat;
pub mod config;
pub mod database;
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

use crate::{chat::ChatState, config::ServiceConfig, database::Database};

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

pub async fn app_with_config(config: ServiceConfig) -> Result<Router, sqlx::Error> {
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
    let database = Database::connect(&config.database_url).await?;
    let user_api_routes = user::api_routes(database.clone());
    let calendar_api_routes = calendar::api_routes(database.clone());
    let chat_state = ChatState::new(
        config.openai_api_key,
        config.openai_model,
        config.openai_base_url,
        database,
    );

    let chat_routes = Router::new()
        .route("/v1/chat/stream", post(chat::stream_chat))
        .route("/v1/conversations", get(chat::list_conversations))
        .route(
            "/v1/conversations/{id}/messages",
            get(chat::conversation_messages),
        )
        .with_state(chat_state);

    Ok(base_router()
        .merge(chat_routes)
        .merge(user_api_routes)
        .merge(calendar_api_routes)
        .layer(cors))
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
    async fn saving_a_user_requires_authentication() {
        let app = app_with_config(test_config("http://127.0.0.1:1".to_owned()))
            .await
            .unwrap();

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
    }

    #[tokio::test]
    async fn saving_a_user_upserts_the_profile() {
        let app = app_with_config(test_config("http://127.0.0.1:1".to_owned()))
            .await
            .unwrap();

        let save = |name: &str| {
            Request::post("/v1/users")
                .header(header::AUTHORIZATION, "Bearer development-token")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": "user-1",
                        "name": name,
                        "email": "curio@example.com",
                        "avatarUrl": "https://example.com/avatar.png"
                    })
                    .to_string(),
                ))
                .unwrap()
        };

        let created = app.clone().oneshot(save("Curio")).await.unwrap();
        assert_eq!(created.status(), StatusCode::OK);
        let created = created.into_body().collect().await.unwrap().to_bytes();
        let created: Value = serde_json::from_slice(&created).unwrap();
        assert_eq!(created["id"], "user-1");
        assert_eq!(created["name"], "Curio");
        assert_eq!(created["email"], "curio@example.com");
        assert_eq!(created["avatarUrl"], "https://example.com/avatar.png");

        let updated = app.oneshot(save("Curio Renamed")).await.unwrap();
        assert_eq!(updated.status(), StatusCode::OK);
        let updated = updated.into_body().collect().await.unwrap().to_bytes();
        let updated: Value = serde_json::from_slice(&updated).unwrap();
        assert_eq!(updated["name"], "Curio Renamed");
    }

    #[tokio::test]
    async fn saving_a_user_rejects_blank_profiles() {
        let app = app_with_config(test_config("http://127.0.0.1:1".to_owned()))
            .await
            .unwrap();

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

    // ---------------------------------------------------------------- calendar

    async fn calendar_app() -> Router {
        let app = app_with_config(test_config("http://127.0.0.1:1".to_owned()))
            .await
            .unwrap();

        // A calendar event's user_id is a foreign key into users, and a user
        // only gets a row when the profile wizard runs, so the tests create the
        // profiles they own events for.
        for (id, email) in [("user-a", "a@example.com"), ("user-b", "b@example.com")] {
            let created = app
                .clone()
                .oneshot(
                    Request::post("/v1/users")
                        .header(header::AUTHORIZATION, "Bearer development-token")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(
                            json!({ "id": id, "name": id, "email": email }).to_string(),
                        ))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(created.status(), StatusCode::OK);
        }

        app
    }

    fn create_event_request(event: Value) -> Request<Body> {
        Request::post("/v1/calendar/events")
            .header(header::AUTHORIZATION, "Bearer development-token")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(event.to_string()))
            .unwrap()
    }

    fn list_events_request(user_id: &str, view: &str, start: &str, end: &str) -> Request<Body> {
        Request::get(format!(
            "/v1/calendar/events?userId={user_id}&view={view}&start={start}&end={end}"
        ))
        .header(header::AUTHORIZATION, "Bearer development-token")
        .body(Body::empty())
        .unwrap()
    }

    async fn json_body(response: Response) -> Value {
        let body = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&body).unwrap()
    }

    async fn create_event(app: &Router, event: Value) -> Value {
        let response = app
            .clone()
            .oneshot(create_event_request(event))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        json_body(response).await
    }

    async fn list_events(app: &Router, user_id: &str, view: &str, start: &str, end: &str) -> Value {
        let response = app
            .clone()
            .oneshot(list_events_request(user_id, view, start, end))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        json_body(response).await
    }

    fn timed_event(id: &str, user_id: &str, start: &str, end: &str) -> Value {
        json!({
            "id": id,
            "userId": user_id,
            "title": "Design review",
            "allDay": false,
            "startDate": start,
            "endDate": end
        })
    }

    #[tokio::test]
    async fn creating_an_event_requires_authentication() {
        let app = calendar_app().await;

        let response = app
            .oneshot(
                Request::post("/v1/calendar/events")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        timed_event(
                            "event-1",
                            "user-a",
                            "2026-06-22T14:00:00.000Z",
                            "2026-06-22T15:30:00.000Z",
                        )
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn listing_events_requires_authentication() {
        let app = calendar_app().await;

        let response = app
            .oneshot(
                Request::get(concat!(
                    "/v1/calendar/events?userId=user-a&view=day",
                    "&start=2026-06-22T00:00:00.000Z&end=2026-06-23T00:00:00.000Z"
                ))
                .body(Body::empty())
                .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn a_created_event_is_returned_with_the_fields_it_was_sent() {
        let app = calendar_app().await;

        let created = create_event(
            &app,
            json!({
                "id": "event-1",
                "userId": "user-a",
                "title": "  Design review  ",
                "description": "Quarterly",
                "status": "scheduled",
                "priority": "medium",
                "allDay": false,
                "startDate": "2026-06-22T14:00:00.000Z",
                "endDate": "2026-06-22T15:30:00.000Z"
            }),
        )
        .await;

        assert_eq!(created["id"], "event-1");
        assert_eq!(created["userId"], "user-a");
        assert_eq!(created["title"], "Design review");
        assert_eq!(created["status"], "scheduled");
        assert_eq!(created["priority"], "medium");
        assert_eq!(created["allDay"], false);
        assert_eq!(created["startDate"], "2026-06-22T14:00:00.000Z");
        assert_eq!(created["endDate"], "2026-06-22T15:30:00.000Z");
        // The normalized instants drive the query and must never reach a client.
        assert!(created.get("startsAt").is_none());
        assert!(created.get("endsAt").is_none());
    }

    #[tokio::test]
    async fn an_event_created_for_one_user_is_absent_from_another_users_range() {
        let app = calendar_app().await;
        create_event(
            &app,
            timed_event(
                "event-1",
                "user-a",
                "2026-06-22T14:00:00.000Z",
                "2026-06-22T15:30:00.000Z",
            ),
        )
        .await;

        let mine = list_events(
            &app,
            "user-a",
            "day",
            "2026-06-22T00:00:00.000Z",
            "2026-06-23T00:00:00.000Z",
        )
        .await;
        assert_eq!(mine["events"].as_array().unwrap().len(), 1);

        let theirs = list_events(
            &app,
            "user-b",
            "day",
            "2026-06-22T00:00:00.000Z",
            "2026-06-23T00:00:00.000Z",
        )
        .await;
        assert!(theirs["events"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn an_all_day_event_round_trips_as_a_date_only_string() {
        let app = calendar_app().await;
        create_event(
            &app,
            json!({
                "id": "event-1",
                "userId": "user-a",
                "title": "Offsite",
                "allDay": true,
                "startDate": "2026-06-22",
                "endDate": null
            }),
        )
        .await;

        let listed = list_events(
            &app,
            "user-a",
            "month",
            "2026-05-31T07:00:00.000Z",
            "2026-07-01T07:00:00.000Z",
        )
        .await;

        assert_eq!(listed["events"][0]["startDate"], "2026-06-22");
        assert_eq!(listed["events"][0]["allDay"], true);
        assert!(listed["events"][0]["endDate"].is_null());
    }

    #[tokio::test]
    async fn a_multi_day_all_day_span_appears_in_a_range_covering_only_its_middle_day() {
        let app = calendar_app().await;
        create_event(
            &app,
            json!({
                "id": "event-1",
                "userId": "user-a",
                "title": "Conference",
                "allDay": true,
                "startDate": "2026-06-22",
                "endDate": "2026-06-25"
            }),
        )
        .await;

        let middle_day = list_events(
            &app,
            "user-a",
            "day",
            "2026-06-24T00:00:00.000Z",
            "2026-06-25T00:00:00.000Z",
        )
        .await;
        assert_eq!(middle_day["events"].as_array().unwrap().len(), 1);

        // The wire end date is inclusive, so the 25th is still inside the span
        // and the 26th is outside it.
        let last_day = list_events(
            &app,
            "user-a",
            "day",
            "2026-06-25T00:00:00.000Z",
            "2026-06-26T00:00:00.000Z",
        )
        .await;
        assert_eq!(last_day["events"].as_array().unwrap().len(), 1);

        let day_after = list_events(
            &app,
            "user-a",
            "day",
            "2026-06-26T00:00:00.000Z",
            "2026-06-27T00:00:00.000Z",
        )
        .await;
        assert!(day_after["events"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn the_range_is_half_open_at_both_ends() {
        let app = calendar_app().await;
        create_event(
            &app,
            timed_event(
                "event-1",
                "user-a",
                "2026-06-22T14:00:00.000Z",
                "2026-06-22T15:00:00.000Z",
            ),
        )
        .await;

        // Ends exactly at the range start: excluded.
        let before = list_events(
            &app,
            "user-a",
            "day",
            "2026-06-22T15:00:00.000Z",
            "2026-06-23T00:00:00.000Z",
        )
        .await;
        assert!(before["events"].as_array().unwrap().is_empty());

        // Starts exactly at the range end: excluded.
        let after = list_events(
            &app,
            "user-a",
            "day",
            "2026-06-22T00:00:00.000Z",
            "2026-06-22T14:00:00.000Z",
        )
        .await;
        assert!(after["events"].as_array().unwrap().is_empty());

        // Overlapping by a minute: included.
        let overlapping = list_events(
            &app,
            "user-a",
            "day",
            "2026-06-22T14:59:00.000Z",
            "2026-06-23T00:00:00.000Z",
        )
        .await;
        assert_eq!(overlapping["events"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn a_milestone_with_no_end_date_is_returned_in_its_range() {
        let app = calendar_app().await;
        let created = create_event(
            &app,
            json!({
                "id": "event-1",
                "userId": "user-a",
                "title": "Launch",
                "allDay": false,
                "startDate": "2026-06-22T14:00:00.000Z"
            }),
        )
        .await;
        assert!(created["endDate"].is_null());

        let listed = list_events(
            &app,
            "user-a",
            "week",
            "2026-06-21T00:00:00.000Z",
            "2026-06-28T00:00:00.000Z",
        )
        .await;

        assert_eq!(listed["events"].as_array().unwrap().len(), 1);
        assert_eq!(listed["events"][0]["startDate"], "2026-06-22T14:00:00.000Z");
        assert!(listed["events"][0]["endDate"].is_null());
    }

    #[tokio::test]
    async fn events_are_ordered_by_when_they_start() {
        let app = calendar_app().await;
        create_event(
            &app,
            timed_event(
                "event-late",
                "user-a",
                "2026-06-22T16:00:00.000Z",
                "2026-06-22T17:00:00.000Z",
            ),
        )
        .await;
        create_event(
            &app,
            timed_event(
                "event-early",
                "user-a",
                "2026-06-22T09:00:00.000Z",
                "2026-06-22T10:00:00.000Z",
            ),
        )
        .await;

        let listed = list_events(
            &app,
            "user-a",
            "day",
            "2026-06-22T00:00:00.000Z",
            "2026-06-23T00:00:00.000Z",
        )
        .await;

        assert_eq!(listed["events"][0]["id"], "event-early");
        assert_eq!(listed["events"][1]["id"], "event-late");
    }

    #[tokio::test]
    async fn a_range_wider_than_the_view_is_rejected() {
        let app = calendar_app().await;

        let response = app
            .oneshot(list_events_request(
                "user-a",
                "day",
                "2026-06-01T00:00:00.000Z",
                "2026-07-01T00:00:00.000Z",
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(json_body(response).await["error"].is_string());
    }

    #[tokio::test]
    async fn an_unknown_view_is_rejected_by_the_extractor() {
        let app = calendar_app().await;

        let response = app
            .oneshot(list_events_request(
                "user-a",
                "decade",
                "2026-06-22T00:00:00.000Z",
                "2026-06-23T00:00:00.000Z",
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn an_inverted_or_unparseable_range_is_rejected() {
        let app = calendar_app().await;

        let inverted = app
            .clone()
            .oneshot(list_events_request(
                "user-a",
                "day",
                "2026-06-23T00:00:00.000Z",
                "2026-06-22T00:00:00.000Z",
            ))
            .await
            .unwrap();
        assert_eq!(inverted.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let unparseable = app
            .oneshot(list_events_request(
                "user-a",
                "day",
                "yesterday",
                "2026-06-23T00:00:00.000Z",
            ))
            .await
            .unwrap();
        assert_eq!(unparseable.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn a_duplicate_event_id_is_a_conflict() {
        let app = calendar_app().await;
        let event = timed_event(
            "event-1",
            "user-a",
            "2026-06-22T14:00:00.000Z",
            "2026-06-22T15:00:00.000Z",
        );
        create_event(&app, event.clone()).await;

        let response = app.oneshot(create_event_request(event)).await.unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn an_event_without_a_title_is_rejected() {
        let app = calendar_app().await;

        let response = app
            .oneshot(create_event_request(json!({
                "userId": "user-a",
                "title": "   ",
                "allDay": false,
                "startDate": "2026-06-22T14:00:00.000Z"
            })))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            json_body(response).await["error"],
            "An event needs a title."
        );
    }

    #[tokio::test]
    async fn an_event_for_a_user_without_a_profile_is_rejected_rather_than_failing() {
        let app = calendar_app().await;

        let response = app
            .oneshot(create_event_request(timed_event(
                "event-1",
                "user-who-never-finished-setup",
                "2026-06-22T14:00:00.000Z",
                "2026-06-22T15:00:00.000Z",
            )))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            json_body(response).await["error"],
            "No profile exists for that user yet."
        );
    }

    #[tokio::test]
    async fn a_start_date_that_disagrees_with_all_day_is_rejected() {
        let app = calendar_app().await;

        let response = app
            .oneshot(create_event_request(json!({
                "userId": "user-a",
                "title": "Offsite",
                "allDay": true,
                "startDate": "2026-06-22T14:00:00.000Z"
            })))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn an_unknown_status_or_priority_is_rejected() {
        let app = calendar_app().await;

        let status = app
            .clone()
            .oneshot(create_event_request(json!({
                "userId": "user-a",
                "title": "Design review",
                "status": "procrastinating",
                "allDay": false,
                "startDate": "2026-06-22T14:00:00.000Z"
            })))
            .await
            .unwrap();
        assert_eq!(status.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let priority = app
            .oneshot(create_event_request(json!({
                "userId": "user-a",
                "title": "Design review",
                "priority": "catastrophic",
                "allDay": false,
                "startDate": "2026-06-22T14:00:00.000Z"
            })))
            .await
            .unwrap();
        assert_eq!(priority.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn an_event_created_without_an_id_gets_one() {
        let app = calendar_app().await;

        let created = create_event(
            &app,
            json!({
                "userId": "user-a",
                "title": "Design review",
                "allDay": false,
                "startDate": "2026-06-22T14:00:00.000Z"
            }),
        )
        .await;

        assert!(!created["id"].as_str().unwrap().is_empty());
    }
}
