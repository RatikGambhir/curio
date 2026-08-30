//! Creating events and reading them back: what the endpoints return, what they
//! keep verbatim, and whose events a range query can see.

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::calendar::{calendar_app, create_event, events_in, timed_event};
use serde_json::json;
use tower::ServiceExt;

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

    let mine = events_in(
        &app,
        "user-a",
        "day",
        "2026-06-22T00:00:00.000Z",
        "2026-06-23T00:00:00.000Z",
    )
    .await;
    assert_eq!(mine.len(), 1);

    let theirs = events_in(
        &app,
        "user-b",
        "day",
        "2026-06-22T00:00:00.000Z",
        "2026-06-23T00:00:00.000Z",
    )
    .await;
    assert!(theirs.is_empty());
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

    let listed = events_in(
        &app,
        "user-a",
        "month",
        "2026-05-31T07:00:00.000Z",
        "2026-07-01T07:00:00.000Z",
    )
    .await;

    assert_eq!(listed[0]["startDate"], "2026-06-22");
    assert_eq!(listed[0]["allDay"], true);
    assert!(listed[0]["endDate"].is_null());
}

#[tokio::test]
async fn a_milestone_keeps_its_missing_end_date() {
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

    let listed = events_in(
        &app,
        "user-a",
        "week",
        "2026-06-21T00:00:00.000Z",
        "2026-06-28T00:00:00.000Z",
    )
    .await;

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["startDate"], "2026-06-22T14:00:00.000Z");
    assert!(listed[0]["endDate"].is_null());
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

    let listed = events_in(
        &app,
        "user-a",
        "day",
        "2026-06-22T00:00:00.000Z",
        "2026-06-23T00:00:00.000Z",
    )
    .await;

    assert_eq!(listed[0]["id"], "event-early");
    assert_eq!(listed[1]["id"], "event-late");
}
