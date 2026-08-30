//! Which events a range returns: the half-open overlap semantics, multi-day
//! spans that straddle the bounds, and the span caps each view enforces.

mod common;

use axum::http::StatusCode;
use common::{
    calendar::{calendar_app, create_event, events_in, list_events_request, timed_event},
    json_body,
};
use serde_json::json;
use tower::ServiceExt;

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

    let middle_day = events_in(
        &app,
        "user-a",
        "day",
        "2026-06-24T00:00:00.000Z",
        "2026-06-25T00:00:00.000Z",
    )
    .await;
    assert_eq!(middle_day.len(), 1);

    // The wire end date is inclusive, so the 25th is still inside the span and
    // the 26th is outside it.
    let last_day = events_in(
        &app,
        "user-a",
        "day",
        "2026-06-25T00:00:00.000Z",
        "2026-06-26T00:00:00.000Z",
    )
    .await;
    assert_eq!(last_day.len(), 1);

    let day_after = events_in(
        &app,
        "user-a",
        "day",
        "2026-06-26T00:00:00.000Z",
        "2026-06-27T00:00:00.000Z",
    )
    .await;
    assert!(day_after.is_empty());
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
    let before = events_in(
        &app,
        "user-a",
        "day",
        "2026-06-22T15:00:00.000Z",
        "2026-06-23T00:00:00.000Z",
    )
    .await;
    assert!(before.is_empty());

    // Starts exactly at the range end: excluded.
    let after = events_in(
        &app,
        "user-a",
        "day",
        "2026-06-22T00:00:00.000Z",
        "2026-06-22T14:00:00.000Z",
    )
    .await;
    assert!(after.is_empty());

    // Overlapping by a minute: included.
    let overlapping = events_in(
        &app,
        "user-a",
        "day",
        "2026-06-22T14:59:00.000Z",
        "2026-06-23T00:00:00.000Z",
    )
    .await;
    assert_eq!(overlapping.len(), 1);
}

#[tokio::test]
async fn a_milestone_with_no_end_date_is_returned_in_its_range() {
    let app = calendar_app().await;
    create_event(
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

    let listed = events_in(
        &app,
        "user-a",
        "week",
        "2026-06-21T00:00:00.000Z",
        "2026-06-28T00:00:00.000Z",
    )
    .await;

    assert_eq!(listed.len(), 1);
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
