use crate::{
    calendar::{
        error::CalendarError,
        models::{CalendarView, CreateEventInput, ListEventsQuery},
        repository::CalendarRepository,
        service::CalendarService,
    },
    database::Database,
};

async fn service() -> CalendarService {
    let database = Database::connect("sqlite::memory:").await.unwrap();
    CalendarService::new(CalendarRepository::new(database))
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
async fn create_validation() {
    assert_eq!(
        service().await.create_event(create_input("   ")).await,
        Err(CalendarError::Invalid("An event needs a title."))
    );

    let mut input = create_input("Design review");
    input.status = Some("procrastinating".to_owned());
    assert_eq!(
        service().await.create_event(input).await,
        Err(CalendarError::Invalid("That is not a known event status."))
    );
}

#[tokio::test]
async fn storage_errors() {
    assert_eq!(
        service()
            .await
            .create_event(create_input("Design review"))
            .await,
        Err(CalendarError::UnknownUser)
    );
}

#[tokio::test]
async fn range_validation() {
    assert_eq!(
        service()
            .await
            .events_in_range(range(
                CalendarView::Day,
                "2026-06-01T00:00:00.000Z",
                "2026-07-01T00:00:00.000Z",
            ))
            .await,
        Err(CalendarError::Invalid(
            "That range is wider than the requested view can cover."
        ))
    );

    assert_eq!(
        service()
            .await
            .events_in_range(range(
                CalendarView::Day,
                "2026-06-23T00:00:00.000Z",
                "2026-06-22T00:00:00.000Z",
            ))
            .await,
        Err(CalendarError::Invalid(
            "The range end must fall after the range start."
        ))
    );
}

#[tokio::test]
async fn dst_boundaries() {
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
