//! PostgreSQL adapter for the calendar persistence port.
use crate::adapters::postgres::query::{
    Comparison, Direction, FieldWriter, InsertQuery, SelectQuery, SqlColumn, SqlField,
};
use chrono::{DateTime, Utc};

use crate::{
    adapters::postgres::client::Database,
    domains::calendar::{
        model::{CalendarError, CalendarEvent, EventRange, NewCalendarEvent},
        service::CalendarStore,
    },
};

#[derive(Clone)]
pub struct CalendarRepository {
    database: Database,
}

impl CalendarRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl CalendarStore for CalendarRepository {
    async fn insert(&self, event: NewCalendarEvent<'_>) -> Result<CalendarEvent, CalendarError> {
        use EventColumn::*;
        InsertQuery::new("calendar_events")
            .value(EventField::Id(event.id()))
            .value(EventField::UserId(event.user_id()))
            .value(EventField::Title(event.title()))
            .value(EventField::Description(event.description()))
            .value(EventField::Status(event.status()))
            .value(EventField::Priority(event.priority()))
            .value(EventField::AllDay(event.all_day()))
            .value(EventField::StartDate(event.start_date()))
            .value(EventField::EndDate(event.end_date()))
            .value(EventField::StartsAt(event.starts_at()))
            .value(EventField::EndsAt(event.ends_at()))
            .returning([
                Id,
                UserId,
                Title,
                Description,
                Status,
                Priority,
                AllDay,
                StartDate,
                EndDate,
                CreatedAt,
                UpdatedAt,
            ])
            .build()
            .map_err(|error| storage_error("calendar_insert", error))?
            .build_query_as::<CalendarEventRow>()
            .fetch_one(self.database.pool())
            .await
            .map(Into::into)
            .map_err(|error| storage_error("calendar_insert", error))
    }

    async fn in_range(&self, range: EventRange<'_>) -> Result<Vec<CalendarEvent>, CalendarError> {
        SelectQuery::new(
            "calendar_events",
            [
                EventColumn::Id,
                EventColumn::UserId,
                EventColumn::Title,
                EventColumn::Description,
                EventColumn::Status,
                EventColumn::Priority,
                EventColumn::AllDay,
                EventColumn::StartDate,
                EventColumn::EndDate,
                EventColumn::CreatedAt,
                EventColumn::UpdatedAt,
            ],
        )
        .filter(EventColumn::UserId, Comparison::Equal, range.user_id())
        .filter(EventColumn::StartsAt, Comparison::Less, range.end())
        .filter(EventColumn::EndsAt, Comparison::Greater, range.start())
        .order_by(EventColumn::StartsAt, Direction::Ascending)
        .order_by(EventColumn::Id, Direction::Ascending)
        .build()
        .map_err(|error| storage_error("calendar_list", error))?
        .build_query_as::<CalendarEventRow>()
        .fetch_all(self.database.pool())
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(|error| storage_error("calendar_list", error))
    }
}

#[derive(sqlx::FromRow)]
struct CalendarEventRow {
    id: String,
    user_id: String,
    title: String,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    all_day: bool,
    start_date: String,
    end_date: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<CalendarEventRow> for CalendarEvent {
    fn from(row: CalendarEventRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            title: row.title,
            description: row.description,
            status: row.status,
            priority: row.priority,
            all_day: row.all_day,
            start_date: row.start_date,
            end_date: row.end_date,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

fn storage_error(context: &'static str, error: sqlx::Error) -> CalendarError {
    if let sqlx::Error::Database(ref database_error) = error {
        if database_error.is_unique_violation() {
            return CalendarError::Conflict("An event with that id already exists.");
        }
        if database_error.is_foreign_key_violation() {
            return CalendarError::UnknownUser;
        }
    }

    crate::shared::diagnostics::log_database_error(context, &error);
    CalendarError::Internal
}

#[derive(Clone, Copy)]
enum EventColumn {
    Id,
    UserId,
    Title,
    Description,
    Status,
    Priority,
    AllDay,
    StartDate,
    EndDate,
    StartsAt,
    EndsAt,
    CreatedAt,
    UpdatedAt,
}
impl SqlColumn for EventColumn {
    fn sql(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::UserId => "user_id",
            Self::StartsAt => "starts_at",
            Self::EndsAt => "ends_at",
            Self::Title => "title",
            Self::Description => "description",
            Self::Status => "status",
            Self::Priority => "priority",
            Self::AllDay => "all_day",
            Self::StartDate => "start_date",
            Self::EndDate => "end_date",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }
}

/// Typed write fields: a variant owns its column mapping and payload type.
enum EventField<'a> {
    Id(&'a str),
    UserId(&'a str),
    Title(&'a str),
    Description(Option<&'a str>),
    Status(Option<&'a str>),
    Priority(Option<&'a str>),
    AllDay(bool),
    StartDate(&'a str),
    EndDate(Option<&'a str>),
    StartsAt(DateTime<Utc>),
    EndsAt(DateTime<Utc>),
}
impl SqlField for EventField<'_> {
    type Column = EventColumn;
    fn write(self, writer: &mut impl FieldWriter<EventColumn>) {
        match self {
            Self::Id(value) => writer.bind(EventColumn::Id, value),
            Self::UserId(value) => writer.bind(EventColumn::UserId, value),
            Self::Title(value) => writer.bind(EventColumn::Title, value),
            Self::Description(value) => writer.bind(EventColumn::Description, value),
            Self::Status(value) => writer.bind(EventColumn::Status, value),
            Self::Priority(value) => writer.bind(EventColumn::Priority, value),
            Self::AllDay(value) => writer.bind(EventColumn::AllDay, value),
            Self::StartDate(value) => writer.bind(EventColumn::StartDate, value),
            Self::EndDate(value) => writer.bind(EventColumn::EndDate, value),
            Self::StartsAt(value) => writer.bind(EventColumn::StartsAt, value),
            Self::EndsAt(value) => writer.bind(EventColumn::EndsAt, value),
        }
    }
}
