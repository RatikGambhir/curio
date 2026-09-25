//! Calendar use cases, composed with an owner-scoped persistence port.
use super::model::{
    CalendarError, CalendarEvent, CreateEventInput, EventRange, ListEventsQuery, NewCalendarEvent,
};

pub trait CalendarStore: Send + Sync {
    fn insert(
        &self,
        event: NewCalendarEvent<'_>,
    ) -> impl Future<Output = Result<CalendarEvent, CalendarError>> + Send;
    fn in_range(
        &self,
        range: EventRange<'_>,
    ) -> impl Future<Output = Result<Vec<CalendarEvent>, CalendarError>> + Send;
}

#[derive(Clone)]
pub struct CalendarService<R> {
    repository: R,
}

impl<R: CalendarStore> CalendarService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn create_event(
        &self,
        input: CreateEventInput,
    ) -> Result<CalendarEvent, CalendarError> {
        self.repository
            .insert(NewCalendarEvent::parse(&input)?)
            .await
    }

    pub async fn events_in_range(
        &self,
        query: ListEventsQuery,
    ) -> Result<Vec<CalendarEvent>, CalendarError> {
        self.repository.in_range(EventRange::parse(&query)?).await
    }
}
