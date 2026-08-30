//! Fixtures for the calendar endpoints.
#![allow(dead_code)]

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use curio_service::app_with_config;
use serde_json::{Value, json};
use tower::ServiceExt;

use super::{TEST_TOKEN, authenticated, json_body, offline_config};

/// An app with two profiles already saved.
///
/// A calendar event's `user_id` is a foreign key into `users`, and a user only
/// gets a row when the profile wizard runs, so the tests create the profiles
/// they own events for. `user-b` exists so scoping has something to be scoped
/// away from.
pub async fn calendar_app() -> Router {
    let app = app_with_config(offline_config()).await.unwrap();

    for (id, email) in [("user-a", "a@example.com"), ("user-b", "b@example.com")] {
        let created = app
            .clone()
            .oneshot(authenticated(
                Request::post("/v1/users"),
                json!({ "id": id, "name": id, "email": email }),
            ))
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::OK);
    }

    app
}

pub fn create_event_request(event: Value) -> Request<Body> {
    authenticated(Request::post("/v1/calendar/events"), event)
}

pub fn list_events_request(user_id: &str, view: &str, start: &str, end: &str) -> Request<Body> {
    Request::get(format!(
        "/v1/calendar/events?userId={user_id}&view={view}&start={start}&end={end}"
    ))
    .header(header::AUTHORIZATION, TEST_TOKEN)
    .body(Body::empty())
    .unwrap()
}

/// Creates an event, asserting it was accepted, and returns the stored record.
pub async fn create_event(app: &Router, event: Value) -> Value {
    let response = app
        .clone()
        .oneshot(create_event_request(event))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    json_body(response).await
}

pub async fn list_events(app: &Router, user_id: &str, view: &str, start: &str, end: &str) -> Value {
    let response = app
        .clone()
        .oneshot(list_events_request(user_id, view, start, end))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await
}

/// The events a range query returns, for the common case of counting them.
pub async fn events_in(
    app: &Router,
    user_id: &str,
    view: &str,
    start: &str,
    end: &str,
) -> Vec<Value> {
    list_events(app, user_id, view, start, end).await["events"]
        .as_array()
        .unwrap()
        .clone()
}

pub fn timed_event(id: &str, user_id: &str, start: &str, end: &str) -> Value {
    json!({
        "id": id,
        "userId": user_id,
        "title": "Design review",
        "allDay": false,
        "startDate": start,
        "endDate": end
    })
}
