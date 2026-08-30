//! Persistence for calendar events. SQL and row mapping live here and nowhere
//! else; this layer applies no rules and returns `sqlx::Error` unchanged, so
//! deciding what a constraint violation means to a client stays in the service.

use sqlx::{Row, SqlitePool, sqlite::SqliteRow};

use crate::calendar::models::{CalendarEvent, NewCalendarEvent};

/// The wire fields, and only the wire fields. `starts_at` / `ends_at` are
/// written and matched against, never selected — naming the list once keeps the
/// two read paths from drifting apart.
const EVENT_COLUMNS: &str = "id, user_id, title, description, status, priority, \
                             all_day, start_date, end_date, created_at, updated_at";

#[derive(Clone)]
pub struct CalendarRepository {
    pool: SqlitePool,
}

impl CalendarRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
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
        .execute(&self.pool)
        .await?;

        let row = sqlx::query(&format!(
            "SELECT {EVENT_COLUMNS} FROM calendar_events WHERE id = ?1"
        ))
        .bind(event.id)
        .fetch_one(&self.pool)
        .await?;

        calendar_event(&row)
    }

    /// Every event overlapping the half-open range `[start, end)`.
    ///
    /// The predicate is a single expression with no special cases because every
    /// stored row satisfies `ends_at > starts_at`, which is what the service's
    /// nominal milestone duration buys. An event ending exactly at `start` and
    /// one starting exactly at `end` are both outside the range.
    pub async fn in_range(
        &self,
        user_id: &str,
        start: &str,
        end: &str,
    ) -> Result<Vec<CalendarEvent>, sqlx::Error> {
        let rows = sqlx::query(&format!(
            r#"
            SELECT {EVENT_COLUMNS}
            FROM calendar_events
            WHERE user_id = ?1 AND starts_at < ?3 AND ends_at > ?2
            ORDER BY starts_at, id
            "#
        ))
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
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
