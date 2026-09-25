//! PostgreSQL profile adapter.
use chrono::{DateTime, Utc};
use sqlx::FromRow;

use super::{
    domain::{UserError, UserProfile, UserRecord},
    service::UserStore,
};
use crate::{
    database::Database,
    query::{Assignment, FieldWriter, InsertQuery, SqlColumn, SqlField},
};

#[derive(Clone)]
pub struct PostgresUserRepository {
    database: Database,
}

impl PostgresUserRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}
impl UserStore for PostgresUserRepository {
    async fn save(&self, profile: UserProfile<'_>) -> Result<UserRecord, UserError> {
        use UserColumn::*;
        InsertQuery::new("users")
            .value(UserField::Id(profile.id()))
            .value(UserField::Name(profile.name()))
            .value(UserField::Email(profile.email()))
            .value(UserField::AvatarUrl(profile.avatar_url()))
            .on_conflict(
                Id,
                [
                    Assignment::Excluded(Name),
                    Assignment::Excluded(Email),
                    Assignment::Excluded(AvatarUrl),
                    Assignment::CurrentTimestamp(UpdatedAt),
                ],
            )
            .returning([Id, Name, Email, AvatarUrl, CreatedAt, UpdatedAt])
            .build()
            .map_err(storage_error)?
            .build_query_as::<UserRow>()
            .fetch_one(self.database.pool())
            .await
            .map(Into::into)
            .map_err(storage_error)
    }
}

#[derive(Debug, FromRow, PartialEq, Eq)]
struct UserRow {
    id: String,
    name: String,
    email: String,
    avatar_url: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<UserRow> for UserRecord {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            email: row.email,
            avatar_url: row.avatar_url,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

fn storage_error(error: sqlx::Error) -> UserError {
    if let sqlx::Error::Database(ref database_error) = error
        && database_error.is_unique_violation()
    {
        return UserError::EmailConflict;
    }
    crate::diagnostics::log_database_error("user_save", &error);
    UserError::Unavailable
}

#[derive(Clone, Copy)]
enum UserColumn {
    Id,
    Name,
    Email,
    AvatarUrl,
    CreatedAt,
    UpdatedAt,
}
impl SqlColumn for UserColumn {
    fn sql(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::Email => "email",
            Self::AvatarUrl => "avatar_url",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }
}

/// Typed write fields: a variant owns its column mapping and payload type.
enum UserField<'a> {
    Id(&'a str),
    Name(&'a str),
    Email(&'a str),
    AvatarUrl(Option<&'a str>),
}
impl SqlField for UserField<'_> {
    type Column = UserColumn;
    fn write(self, writer: &mut impl FieldWriter<UserColumn>) {
        match self {
            Self::Id(value) => writer.bind(UserColumn::Id, value),
            Self::Name(value) => writer.bind(UserColumn::Name, value),
            Self::Email(value) => writer.bind(UserColumn::Email, value),
            Self::AvatarUrl(value) => writer.bind(UserColumn::AvatarUrl, value),
        }
    }
}
