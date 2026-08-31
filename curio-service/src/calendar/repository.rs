use sqlx::{Row, sqlite::SqliteRow};

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
        sqlx::query(
            r#"
            INSERT INTO calendar_events (
                id, user_id, title, description, status, priority,
                all_day, start_date, end_date, starts_at, ends_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(event.id)
        .bind(event.user_id)
        .bind(event.title)
        .bind(event.description)
        .bind(event.status)
        .bind(event.priority)
        .bind(i64::from(event.all_day))
        .bind(event.start_date)
        .bind(event.end_date)
        .bind(event.starts_at)
        .bind(event.ends_at)
        .execute(self.database.pool())
        .await?;

        let row = sqlx::query(
            r#"
            SELECT id, user_id, title, description, status, priority,
                   all_day, start_date, end_date, created_at, updated_at
            FROM calendar_events
            WHERE id = ?1
            "#,
        )
        .bind(event.id)
        .fetch_one(self.database.pool())
        .await?;

        calendar_event(&row)
    }

    pub async fn in_range(
        &self,
        user_id: &str,
        start: &str,
        end: &str,
    ) -> Result<Vec<CalendarEvent>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, title, description, status, priority,
                   all_day, start_date, end_date, created_at, updated_at
            FROM calendar_events
            WHERE user_id = ?1 AND starts_at < ?3 AND ends_at > ?2
            ORDER BY starts_at, id
            "#,
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_all(self.database.pool())
        .await?;

        rows.iter().map(calendar_event).collect()
    }
}

fn calendar_event(row: &SqliteRow) -> Result<CalendarEvent, sqlx::Error> {
    Ok(CalendarEvent {
        id: row.try_get("id")?,
        user_id: row.try_get("user_id")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        status: row.try_get("status")?,
        priority: row.try_get("priority")?,
        all_day: row.try_get::<i64, _>("all_day")? != 0,
        start_date: row.try_get("start_date")?,
        end_date: row.try_get("end_date")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
