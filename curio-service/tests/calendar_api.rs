use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    response::Response,
};
use curio_service::{app_with_config, config::ServiceConfig};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

const TEST_TOKEN: &str = "Bearer development-token";

async fn calendar_app() -> Router {
    let app = app_with_config(ServiceConfig {
        openai_api_key: "local-test-value".to_owned(),
        openai_model: "configured-test-value".to_owned(),
        openai_base_url: "http://127.0.0.1:1".to_owned(),
        database_url: "sqlite::memory:".to_owned(),
        cors_allowed_origins: vec!["http://localhost:5173".to_owned()],
    })
    .await
    .unwrap();

    for id in ["user-a", "user-b"] {
        let response = app
            .clone()
            .oneshot(authenticated_json(
                Request::post("/v1/users"),
                json!({
                    "id": id,
                    "name": id,
                    "email": format!("{id}@example.com")
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    app
}

fn authenticated_json(builder: axum::http::request::Builder, body: Value) -> Request<Body> {
    builder
        .header(header::AUTHORIZATION, TEST_TOKEN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn create_request(event: Value) -> Request<Body> {
    authenticated_json(Request::post("/v1/calendar/events"), event)
}

fn list_request(user_id: &str, view: &str, start: &str, end: &str) -> Request<Body> {
    Request::get(format!(
        "/v1/calendar/events?userId={user_id}&view={view}&start={start}&end={end}"
    ))
    .header(header::AUTHORIZATION, TEST_TOKEN)
    .body(Body::empty())
    .unwrap()
}

async fn create_event(app: &Router, event: Value) -> Value {
    let response = app.clone().oneshot(create_request(event)).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    json_body(response).await
}

async fn list_events(
    app: &Router,
    user_id: &str,
    view: &str,
    start: &str,
    end: &str,
) -> Vec<Value> {
    let response = app
        .clone()
        .oneshot(list_request(user_id, view, start, end))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await["events"]
        .as_array()
        .unwrap()
        .clone()
}

async fn json_body(response: Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn timed_event(id: &str, user_id: &str, start: &str, end: Option<&str>) -> Value {
    json!({
        "id": id,
        "userId": user_id,
        "title": "Design review",
        "allDay": false,
        "startDate": start,
        "endDate": end
    })
}

mod authentication {
    use super::*;

    #[tokio::test]
    async fn create_and_list_require_a_bearer_token() {
        let app = calendar_app().await;
        let create = app
            .clone()
            .oneshot(
                Request::post("/v1/calendar/events")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        timed_event("event-1", "user-a", "2026-06-22T14:00:00.000Z", None)
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let list = app
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

        assert_eq!(create.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(list.status(), StatusCode::UNAUTHORIZED);
    }
}

mod events {
    use super::*;

    #[tokio::test]
    async fn records_round_trip_without_exposing_normalized_instants() {
        let app = calendar_app().await;
        let created = create_event(
            &app,
            json!({
                "id": "all-day-span",
                "userId": "user-a",
                "title": "  Offsite  ",
                "status": "scheduled",
                "priority": "medium",
                "allDay": true,
                "startDate": "2026-06-22",
                "endDate": "2026-06-25"
            }),
        )
        .await;

        assert_eq!(created["title"], "Offsite");
        assert_eq!(created["startDate"], "2026-06-22");
        assert_eq!(created["endDate"], "2026-06-25");
        assert!(created.get("startsAt").is_none());
        assert!(created.get("endsAt").is_none());
    }

    #[tokio::test]
    async fn generated_ids_and_user_scoping() {
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

        let mine = list_events(
            &app,
            "user-a",
            "day",
            "2026-06-22T00:00:00.000Z",
            "2026-06-23T00:00:00.000Z",
        )
        .await;
        let theirs = list_events(
            &app,
            "user-b",
            "day",
            "2026-06-22T00:00:00.000Z",
            "2026-06-23T00:00:00.000Z",
        )
        .await;

        assert_eq!(mine.len(), 1);
        assert!(theirs.is_empty());
    }
}

mod ranges {
    use super::*;

    #[tokio::test]
    async fn month_week_and_day_include_overlapping_events() {
        let app = calendar_app().await;
        create_event(
            &app,
            json!({
                "id": "all-day-span",
                "userId": "user-a",
                "title": "Offsite",
                "allDay": true,
                "startDate": "2026-06-22",
                "endDate": "2026-06-25"
            }),
        )
        .await;

        for (view, start, end) in [
            (
                "day",
                "2026-06-23T00:00:00.000Z",
                "2026-06-24T00:00:00.000Z",
            ),
            (
                "week",
                "2026-06-21T00:00:00.000Z",
                "2026-06-28T00:00:00.000Z",
            ),
            (
                "month",
                "2026-05-31T00:00:00.000Z",
                "2026-07-12T00:00:00.000Z",
            ),
        ] {
            let events = list_events(&app, "user-a", view, start, end).await;
            assert_eq!(events.len(), 1);
            assert_eq!(events[0]["id"], "all-day-span");
        }
    }

    #[tokio::test]
    async fn ranges_are_half_open_and_include_milestones() {
        let app = calendar_app().await;
        for event in [
            timed_event(
                "ends-at-start",
                "user-a",
                "2026-06-22T09:00:00.000Z",
                Some("2026-06-22T10:00:00.000Z"),
            ),
            timed_event("milestone", "user-a", "2026-06-22T10:30:00.000Z", None),
            timed_event(
                "starts-at-end",
                "user-a",
                "2026-06-22T11:00:00.000Z",
                Some("2026-06-22T12:00:00.000Z"),
            ),
        ] {
            create_event(&app, event).await;
        }

        let events = list_events(
            &app,
            "user-a",
            "day",
            "2026-06-22T10:00:00.000Z",
            "2026-06-22T11:00:00.000Z",
        )
        .await;

        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["id"], "milestone");
        assert!(events[0]["endDate"].is_null());
    }
}

mod validation {
    use super::*;

    #[tokio::test]
    async fn create_failures_are_structured() {
        let app = calendar_app().await;
        let valid = timed_event("event-1", "user-a", "2026-06-22T14:00:00.000Z", None);
        create_event(&app, valid.clone()).await;

        let duplicate = app.clone().oneshot(create_request(valid)).await.unwrap();
        let blank = app
            .clone()
            .oneshot(create_request(json!({
                "userId": "user-a",
                "title": "  ",
                "allDay": false,
                "startDate": "2026-06-22T14:00:00.000Z"
            })))
            .await
            .unwrap();
        let unknown_user = app
            .clone()
            .oneshot(create_request(timed_event(
                "missing-user-event",
                "missing-user",
                "2026-06-22T14:00:00.000Z",
                None,
            )))
            .await
            .unwrap();
        let bad_status = app
            .oneshot(create_request(json!({
                "userId": "user-a",
                "title": "Design review",
                "status": "procrastinating",
                "allDay": false,
                "startDate": "2026-06-22T14:00:00.000Z"
            })))
            .await
            .unwrap();

        assert_eq!(duplicate.status(), StatusCode::CONFLICT);
        assert_eq!(blank.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(unknown_user.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(bad_status.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(json_body(duplicate).await["error"].is_string());
        assert!(json_body(blank).await["error"].is_string());
        assert_eq!(
            json_body(unknown_user).await["error"],
            "No profile exists for that user yet."
        );
        assert!(json_body(bad_status).await["error"].is_string());
    }

    #[tokio::test]
    async fn list_failures_use_the_expected_statuses() {
        let app = calendar_app().await;
        let too_wide = app
            .clone()
            .oneshot(list_request(
                "user-a",
                "day",
                "2026-06-01T00:00:00.000Z",
                "2026-07-01T00:00:00.000Z",
            ))
            .await
            .unwrap();
        let unknown_view = app
            .oneshot(list_request(
                "user-a",
                "quarter",
                "2026-06-01T00:00:00.000Z",
                "2026-06-02T00:00:00.000Z",
            ))
            .await
            .unwrap();

        assert_eq!(too_wide.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(json_body(too_wide).await["error"].is_string());
        assert_eq!(unknown_view.status(), StatusCode::BAD_REQUEST);
    }
}
