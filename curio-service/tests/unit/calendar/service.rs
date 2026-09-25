use crate::{
    calendar::{
        domain::{
            CalendarError, CalendarEvent, CalendarView, CreateEventInput, EventRange,
            ListEventsQuery, NewCalendarEvent,
        },
        repository::PostgresCalendarRepository,
        service::{CalendarService, CalendarStore},
    },
    postgres_test_support::PostgresFixture,
};

async fn service(
    test_name: &str,
) -> Option<(CalendarService<PostgresCalendarRepository>, PostgresFixture)> {
    let postgres = PostgresFixture::provision(test_name).await?;
    let service =
        CalendarService::new(PostgresCalendarRepository::new(postgres.database().clone()));
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
    let service = CalendarService::new(EmptyStore);
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
    let service = CalendarService::new(EmptyStore);
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
}

#[tokio::test]
async fn dst_boundaries() {
    let service = CalendarService::new(EmptyStore);
    let events = service
        .events_in_range(range(
            CalendarView::Day,
            "2026-11-01T07:00:00.000Z",
            "2026-11-02T08:00:00.000Z",
        ))
        .await;

    assert_eq!(events, Ok(Vec::new()));
}

/// Pure validation tests must never acquire a database connection.
struct EmptyStore;
impl CalendarStore for EmptyStore {
    async fn insert(&self, _: NewCalendarEvent<'_>) -> Result<CalendarEvent, CalendarError> {
        panic!("invalid events must not reach persistence")
    }
    async fn in_range(&self, _: EventRange<'_>) -> Result<Vec<CalendarEvent>, CalendarError> {
        Ok(Vec::new())
    }
}

#[test]
fn validated_event_preserves_trimmed_wire_values_and_normalizes_query_interval() {
    let mut input = create_input("  Planning  ");
    input.id = Some(" event-id ".into());
    input.user_id = " owner ".into();
    input.start_date = " 2026-06-22T09:00:00-05:00 ".into();
    input.description = Some("  ".into());
    input.status = Some(" scheduled ".into());
    input.priority = Some(" high ".into());
    let event = NewCalendarEvent::parse(&input).unwrap();
    assert_eq!(event.id(), "event-id");
    assert_eq!(event.user_id(), "owner");
    assert_eq!(event.title(), "Planning");
    assert_eq!(event.start_date(), "2026-06-22T09:00:00-05:00");
    assert_eq!(event.end_date(), None);
    assert_eq!(event.description(), None);
    assert_eq!(event.status(), Some("scheduled"));
    assert_eq!(event.priority(), Some("high"));
    assert_eq!(event.starts_at().to_rfc3339(), "2026-06-22T14:00:00+00:00");
    assert_eq!(
        event.ends_at() - event.starts_at(),
        chrono::Duration::minutes(1)
    );
}
