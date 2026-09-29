//! Bound, owner-scoped PostgreSQL statements for standalone Spaces.
use chrono::{DateTime, Utc};

use super::{
    model::{NewSpace, Space, SpaceError, SpacePage},
    service::SpaceStore,
};
use crate::adapters::postgres::{
    client::Database,
    query::{Delete, Insert, Select, SqlColumn, col, sql_columns, tuple, val},
};
use crate::domains::users::repository::UserColumn;
sql_columns! { pub(crate) enum SpaceColumn {
    Id => "id", OwnerId => "owner_id", Name => "name", Description => "description",
    CreatedAt => "created_at", UpdatedAt => "updated_at"
} }
const SPACE_COLUMNS: [SpaceColumn; 5] = [
    SpaceColumn::Id,
    SpaceColumn::Name,
    SpaceColumn::Description,
    SpaceColumn::CreatedAt,
    SpaceColumn::UpdatedAt,
];

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
        Insert::into("spaces")
            .value(SpaceColumn::Id.value(space.id()))
            .value(SpaceColumn::OwnerId.value(owner))
            .value(SpaceColumn::Name.value(space.name()))
            .value(SpaceColumn::Description.value(space.description()))
            .returning(SPACE_COLUMNS)
            .build()
            .map_err(|error| storage_error("spaces_insert", error))?
            .build_query_as::<SpaceRow>()
            .fetch_one(self.database.pool())
            .await
            .map(Into::into)
            .map_err(|error| storage_error("spaces_insert", error))
    }

    async fn list(&self, owner: &str, page: &SpacePage) -> Result<Vec<Space>, SpaceError> {
        Select::from("spaces")
            .columns(SPACE_COLUMNS)
            .where_(SpaceColumn::OwnerId.is_equal_to(owner))
            .where_optional(page.cursor().map(|cursor| {
                tuple([col(SpaceColumn::CreatedAt), col(SpaceColumn::Id)])
                    .lt(tuple([val(cursor.created_at()), val(cursor.id())]))
            }))
            .order_by(SpaceColumn::CreatedAt.desc())
            .order_by(SpaceColumn::Id.desc())
            .limit(val((page.limit() + 1) as i64))
            .build()
            .map_err(|error| storage_error("spaces_list", error))?
            .build_query_as::<SpaceRow>()
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
        Select::from("users")
            .columns([UserColumn::Id])
            .where_(UserColumn::Id.is_equal_to(owner))
            .for_update()
            .build()
            .map_err(|error| storage_error("spaces_delete_lock", error))?
            .build()
            .fetch_optional(&mut *tx)
            .await
            .map_err(|error| storage_error("spaces_delete_lock", error))?;
        Select::from("spaces")
            .columns([SpaceColumn::Id])
            .where_(SpaceColumn::OwnerId.is_equal_to(owner))
            .where_(SpaceColumn::Id.is_equal_to(id))
            .for_update()
            .build()
            .map_err(|error| storage_error("spaces_delete_lookup", error))?
            .build_query_scalar::<String>()
            .fetch_optional(&mut *tx)
            .await
            .map_err(|error| storage_error("spaces_delete_lookup", error))?
            .ok_or(SpaceError::NotFound)?;
        crate::domains::tasks::repository::detach_space_tasks(&mut tx, owner, id)
            .await
            .map_err(|error| storage_error("spaces_detach_tasks", error))?;
        Delete::from("spaces")
            .where_(SpaceColumn::Id.is_equal_to(id))
            .where_(SpaceColumn::OwnerId.is_equal_to(owner))
            .returning([SpaceColumn::Id])
            .build()
            .map_err(|error| storage_error("spaces_delete", error))?
            .build_query_scalar::<String>()
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
