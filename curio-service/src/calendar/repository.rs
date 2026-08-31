use chrono::{DateTime, Utc};

use crate::{
    calendar::models::{CalendarEvent, NewCalendarEvent},
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
        sqlx::query_as::<_, CalendarEvent>(
            r#"
            INSERT INTO calendar_events (
                id, user_id, title, description, status, priority,
                all_day, start_date, end_date, starts_at, ends_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, user_id, title, description, status, priority,
                      all_day, start_date, end_date, created_at, updated_at
            "#,
        )
        .bind(event.id)
        .bind(event.user_id)
        .bind(event.title)
        .bind(event.description)
        .bind(event.status)
        .bind(event.priority)
        .bind(event.all_day)
        .bind(event.start_date)
        .bind(event.end_date)
        .bind(event.starts_at)
        .bind(event.ends_at)
        .fetch_one(self.database.pool())
        .await
    }

    pub async fn in_range(
        &self,
        user_id: &str,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>, sqlx::Error> {
        sqlx::query_as::<_, CalendarEvent>(
            r#"
            SELECT id, user_id, title, description, status, priority,
                   all_day, start_date, end_date, created_at, updated_at
            FROM calendar_events
            WHERE user_id = $1 AND starts_at < $3 AND ends_at > $2
            ORDER BY starts_at, id
            "#,
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_all(self.database.pool())
        .await
    }
}
