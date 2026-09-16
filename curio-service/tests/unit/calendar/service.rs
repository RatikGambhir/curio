use crate::{
    calendar::{
        error::CalendarError,
        model::{CalendarView, CreateEventInput, ListEventsQuery},
        repository::CalendarRepository,
        service::CalendarService,
    },
    postgres_test_support::PostgresFixture,
};

async fn service(test_name: &str) -> Option<(CalendarService, PostgresFixture)> {
    let postgres = PostgresFixture::provision(test_name).await?;
    let service = CalendarService::new(CalendarRepository::new(postgres.database().clone()));
    Some((service, postgres))
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
    let Some((service, postgres)) = service("calendar_create_validation").await else {
        return;
    };
    assert_eq!(
        service.create_event(create_input("   ")).await,
        Err(CalendarError::Invalid("An event needs a title."))
    );

    let mut input = create_input("Design review");
    input.status = Some("procrastinating".to_owned());
    assert_eq!(
        service.create_event(input).await,
        Err(CalendarError::Invalid("That is not a known event status."))
    );

    postgres.cleanup().await;
}

#[tokio::test]
async fn storage_errors() {
    let Some((service, postgres)) = service("calendar_storage_errors").await else {
        return;
    };
    assert_eq!(
        service.create_event(create_input("Design review")).await,
        Err(CalendarError::UnknownUser)
    );

    postgres.cleanup().await;
}

#[tokio::test]
async fn range_validation() {
    let Some((service, postgres)) = service("calendar_range_validation").await else {
        return;
    };
    assert_eq!(
        service
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
        service
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

    postgres.cleanup().await;
}

#[tokio::test]
async fn dst_boundaries() {
    let Some((service, postgres)) = service("calendar_dst_boundaries").await else {
        return;
    };
    let events = service
        .events_in_range(range(
            CalendarView::Day,
            "2026-11-01T07:00:00.000Z",
            "2026-11-02T08:00:00.000Z",
        ))
        .await;

    assert_eq!(events, Ok(Vec::new()));
    postgres.cleanup().await;
}
