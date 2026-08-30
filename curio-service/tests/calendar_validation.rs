//! Every way a calendar request can be refused, and the status it earns.
//!
//! The point of the file is that none of these is a bare `500`: each one comes
//! back through `CalendarError` as `{"error": "…"}`, which is what the web
//! client already knows how to display.

mod common;

use axum::http::StatusCode;
use common::{
    calendar::{
        calendar_app, create_event, create_event_request, list_events_request, timed_event,
    },
    json_body,
};
use serde_json::json;
use tower::ServiceExt;

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

/// The one rejection that never reaches the handler: `view` is an enum, so
/// serde fails it in the `Query` extractor and Axum answers `400` before any of
/// the calendar's own code runs.
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
