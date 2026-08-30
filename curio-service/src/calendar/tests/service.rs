//! The service's rules, reached directly rather than through the router.
//!
//! These assert the `CalendarError` variant a bad request produces, where the
//! HTTP tests in `tests/calendar_validation.rs` assert the status it turns
//! into. Splitting them that way means a change to the status mapping cannot
//! quietly pass by changing which rule fired.

use crate::calendar::{
    error::CalendarError,
    models::{CalendarView, CreateEventInput, ListEventsQuery},
    repository::CalendarRepository,
    service::CalendarService,
};
use crate::database::Database;

/// The rules are reachable without a router, which is the point of keeping
/// them out of the handlers. A real pool still backs the service so the
/// storage-error mapping is exercised by the same tests.
async fn service() -> CalendarService {
    let database = Database::connect("sqlite::memory:").await.unwrap();
    CalendarService::new(CalendarRepository::new(database.pool().clone()))
}

fn create_input(title: &str) -> CreateEventInput {
    CreateEventInput {
        id: None,
        user_id: "user-a".to_owned(),
        title: title.to_owned(),
        description: None,
        status: None,
        priority: None,
        all_day: false,
        start_date: "2026-06-22T14:00:00.000Z".to_owned(),
        end_date: None,
    }
}

fn range(view: CalendarView, start: &str, end: &str) -> ListEventsQuery {
    ListEventsQuery {
        user_id: "user-a".to_owned(),
        view,
        start: start.to_owned(),
        end: end.to_owned(),
    }
}

#[tokio::test]
async fn a_blank_title_never_reaches_storage() {
    let error = service().await.create_event(create_input("   ")).await;

    assert_eq!(
        error,
        Err(CalendarError::Invalid("An event needs a title."))
    );
}

#[tokio::test]
async fn an_unknown_status_is_rejected_before_the_insert() {
    let mut input = create_input("Design review");
    input.status = Some("procrastinating".to_owned());

    assert_eq!(
        service().await.create_event(input).await,
        Err(CalendarError::Invalid("That is not a known event status."))
    );
}

#[tokio::test]
async fn an_event_for_a_user_without_a_profile_is_an_unknown_user() {
    // Nothing created a users row, so the foreign key is what rejects this.
    let error = service()
        .await
        .create_event(create_input("Design review"))
        .await;

    assert_eq!(error, Err(CalendarError::UnknownUser));
}

#[tokio::test]
async fn a_range_wider_than_its_view_is_rejected() {
    let error = service()
        .await
        .events_in_range(range(
            CalendarView::Day,
            "2026-06-01T00:00:00.000Z",
            "2026-07-01T00:00:00.000Z",
        ))
        .await;

    assert_eq!(
        error,
        Err(CalendarError::Invalid(
            "That range is wider than the requested view can cover."
        ))
    );
}

#[tokio::test]
async fn a_days_worth_of_dst_slack_is_still_a_day() {
    // A local day that crosses a DST boundary is 25 hours once converted to
    // UTC, and must not be mistaken for an oversized range.
    let events = service()
        .await
        .events_in_range(range(
            CalendarView::Day,
            "2026-11-01T07:00:00.000Z",
            "2026-11-02T08:00:00.000Z",
        ))
        .await;

    assert_eq!(events, Ok(Vec::new()));
}

#[tokio::test]
async fn an_inverted_range_is_rejected() {
    let error = service()
        .await
        .events_in_range(range(
            CalendarView::Day,
            "2026-06-23T00:00:00.000Z",
            "2026-06-22T00:00:00.000Z",
        ))
        .await;

    assert_eq!(
        error,
        Err(CalendarError::Invalid(
            "The range end must fall after the range start."
        ))
    );
}
