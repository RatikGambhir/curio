use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

use crate::database::Database;

#[derive(Clone)]
pub struct UserRepository {
    database: Database,
}

#[derive(Debug, FromRow, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserRecord {
    pub id: String,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

impl UserRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn save(
        &self,
        id: &str,
        name: &str,
        email: &str,
        avatar_url: Option<&str>,
    ) -> Result<UserRecord, sqlx::Error> {
        sqlx::query_as::<_, UserRecord>(
            r#"
            INSERT INTO users (id, name, email, avatar_url)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                email = EXCLUDED.email,
                avatar_url = EXCLUDED.avatar_url,
                updated_at = CURRENT_TIMESTAMP
            RETURNING id, name, email, avatar_url, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(email)
        .bind(avatar_url)
        .fetch_one(self.database.pool())
        .await
    }
}
