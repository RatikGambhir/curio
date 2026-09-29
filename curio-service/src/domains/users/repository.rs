//! PostgreSQL profile adapter.
use chrono::{DateTime, Utc};
use sqlx::FromRow;

use super::{
    model::{UserError, UserProfile, UserRecord},
    service::UserStore,
};
use crate::{
    adapters::postgres::client::Database,
    adapters::postgres::query::{Insert, SqlColumn, SqlField, WriteField, current_timestamp},
};

#[derive(Clone)]
pub struct UserRepository {
    database: Database,
}

impl UserRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}
impl UserStore for UserRepository {
    async fn save(&self, profile: UserProfile<'_>) -> Result<UserRecord, UserError> {
        use UserColumn::*;
        Insert::into("users")
            .value(UserField::Id(profile.id()))
            .value(UserField::Name(profile.name()))
            .value(UserField::Email(profile.email()))
            .value(UserField::AvatarUrl(profile.avatar_url()))
            .on_conflict(
                Id,
                [
                    (Name, Name.of("EXCLUDED")),
                    (Email, Email.of("EXCLUDED")),
                    (AvatarUrl, AvatarUrl.of("EXCLUDED")),
                    (UpdatedAt, current_timestamp()),
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
    crate::shared::diagnostics::log_database_error("user_save", &error);
    UserError::Unavailable
}

#[derive(Clone, Copy)]
pub(crate) enum UserColumn {
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
impl<'a> SqlField<'a> for UserField<'a> {
    fn into_field(self) -> WriteField<'a> {
        match self {
            Self::Id(value) => UserColumn::Id.value(value),
            Self::Name(value) => UserColumn::Name.value(value),
            Self::Email(value) => UserColumn::Email.value(value),
            Self::AvatarUrl(value) => UserColumn::AvatarUrl.value(value),
        }
    }
}
