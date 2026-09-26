//! Bound, owner-scoped PostgreSQL statements for standalone Spaces.
use chrono::{DateTime, Utc};

use super::{
    model::{NewSpace, Space, SpaceError, SpacePage},
    service::SpaceStore,
};
use crate::adapters::postgres::client::Database;

pub struct SpaceRepository {
    database: Database,
}

impl SpaceRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl SpaceStore for SpaceRepository {
    async fn insert(&self, owner: &str, space: NewSpace<'_>) -> Result<Space, SpaceError> {
        sqlx::query_as::<_, SpaceRow>(
            "INSERT INTO spaces (id, owner_id, name, description) VALUES ($1, $2, $3, $4)
             RETURNING id, name, description, created_at, updated_at",
        )
        .bind(space.id())
        .bind(owner)
        .bind(space.name())
        .bind(space.description())
        .fetch_one(self.database.pool())
        .await
        .map(Into::into)
        .map_err(|error| storage_error("spaces_insert", error))
    }

    async fn list(&self, owner: &str, page: &SpacePage) -> Result<Vec<Space>, SpaceError> {
        let cursor = page.cursor();
        sqlx::query_as::<_, SpaceRow>(
            "SELECT id, name, description, created_at, updated_at FROM spaces
             WHERE owner_id = $1
               AND ($2::timestamptz IS NULL OR (created_at, id) < ($2, $3))
             ORDER BY created_at DESC, id DESC LIMIT $4",
        )
        .bind(owner)
        .bind(cursor.map(|value| value.created_at()))
        .bind(cursor.map(|value| value.id()))
        .bind((page.limit() + 1) as i64)
        .fetch_all(self.database.pool())
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(|error| storage_error("spaces_list", error))
    }

    async fn delete(&self, owner: &str, id: &str) -> Result<(), SpaceError> {
        let mut tx = self
            .database
            .pool()
            .begin()
            .await
            .map_err(|error| storage_error("spaces_delete_begin", error))?;
        // Share the Tasks lock order so creates/reparents cannot race detachment.
        sqlx::query("SELECT id FROM users WHERE id=$1 FOR UPDATE")
            .bind(owner)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|error| storage_error("spaces_delete_lock", error))?;
        sqlx::query_scalar::<_, String>(
            "SELECT id FROM spaces WHERE owner_id=$1 AND id=$2 FOR UPDATE",
        )
        .bind(owner)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|error| storage_error("spaces_delete_lookup", error))?
        .ok_or(SpaceError::NotFound)?;
        crate::domains::tasks::repository::detach_space_tasks(&mut tx, owner, id)
            .await
            .map_err(|error| storage_error("spaces_detach_tasks", error))?;
        sqlx::query_scalar::<_, String>(
            "DELETE FROM spaces WHERE id=$1 AND owner_id=$2 RETURNING id",
        )
        .bind(id)
        .bind(owner)
        .fetch_one(&mut *tx)
        .await
        .map_err(|error| storage_error("spaces_delete", error))?;
        tx.commit()
            .await
            .map_err(|error| storage_error("spaces_delete_commit", error))
    }
}

#[derive(sqlx::FromRow)]
struct SpaceRow {
    id: String,
    name: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SpaceRow> for Space {
    fn from(row: SpaceRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

fn storage_error(context: &'static str, error: sqlx::Error) -> SpaceError {
    if let sqlx::Error::Database(ref database_error) = error {
        if database_error.is_unique_violation()
            && database_error.constraint() == Some("spaces_owner_name_ci_idx")
        {
            return SpaceError::DuplicateName;
        }
        if database_error.is_foreign_key_violation()
            && database_error.constraint() == Some("spaces_owner_id_fkey")
        {
            return SpaceError::UnknownUser;
        }
    }
    crate::shared::diagnostics::log_database_error(context, &error);
    SpaceError::Internal
}
