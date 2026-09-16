use chrono::{DateTime, Utc};

use crate::{
    calendar::model::{CalendarEvent, NewCalendarEvent},
    core::sql::Sql,
    database::Database,
};

#[derive(Clone)]
pub struct CalendarRepository {
    database: Database,
}

impl CalendarRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn insert(&self, event: NewCalendarEvent<'_>) -> Result<CalendarEvent, sqlx::Error> {
        Sql::insert_into("calendar_events")
            .set("id", event.id)
            .set("user_id", event.user_id)
            .set("title", event.title)
            .set("description", event.description)
            .set("status", event.status)
            .set("priority", event.priority)
            .set("all_day", event.all_day)
            .set("start_date", event.start_date)
            .set("end_date", event.end_date)
            .set("starts_at", event.starts_at)
            .set("ends_at", event.ends_at)
            .returning(CalendarEvent::COLUMNS)
            .fetch_one(self.database.pool())
            .await
    }

    pub async fn in_range(
        &self,
        user_id: &str,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>, sqlx::Error> {
        Sql::select(CalendarEvent::COLUMNS)
            .from("calendar_events")
            .filter("user_id = ?", user_id)
            .filter("starts_at < ?", end)
            .filter("ends_at > ?", start)
            .order_by("starts_at, id")
            .fetch_all(self.database.pool())
            .await
    }
}
