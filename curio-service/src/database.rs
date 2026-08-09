use std::str::FromStr;

use serde::Serialize;
use sqlx::{
    Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

use crate::chat::protocol::ChatStreamRequest;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConversationRecord {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MessageRecord {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub status: String,
    pub response_id: Option<String>,
    pub error_code: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Database {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
        let max_connections = if database_url.contains(":memory:") {
            1
        } else {
            5
        };
        let pool = SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect_with(options)
            .await?;

        sqlx::migrate!().run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn begin_chat(&self, request: &ChatStreamRequest) -> Result<(), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO conversations (id)
            VALUES (?1)
            ON CONFLICT(id) DO UPDATE SET updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&request.conversation_id)
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO messages (id, conversation_id, role, content, status)
            VALUES (?1, ?2, 'user', ?3, 'completed')
            "#,
        )
        .bind(&request.user_message_id)
        .bind(&request.conversation_id)
        .bind(&request.prompt)
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO messages (id, conversation_id, role, content, status)
            VALUES (?1, ?2, 'assistant', '', 'pending')
            "#,
        )
        .bind(&request.assistant_message_id)
        .bind(&request.conversation_id)
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await
    }

    pub async fn complete_assistant(
        &self,
        request: &ChatStreamRequest,
        content: &str,
        response_id: &str,
    ) -> Result<(), sqlx::Error> {
        self.finish_assistant(request, content, "completed", Some(response_id), None)
            .await
    }

    pub async fn fail_assistant(
        &self,
        request: &ChatStreamRequest,
        content: &str,
        error_code: &str,
    ) -> Result<(), sqlx::Error> {
        self.finish_assistant(request, content, "failed", None, Some(error_code))
            .await
    }

    pub async fn interrupt_assistant(
        &self,
        request: &ChatStreamRequest,
        content: &str,
        error_code: &str,
    ) -> Result<(), sqlx::Error> {
        self.finish_assistant(request, content, "interrupted", None, Some(error_code))
            .await
    }

    async fn finish_assistant(
        &self,
        request: &ChatStreamRequest,
        content: &str,
        status: &str,
        response_id: Option<&str>,
        error_code: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        let result = sqlx::query(
            r#"
            UPDATE messages
            SET content = ?1,
                status = ?2,
                response_id = ?3,
                error_code = ?4,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?5 AND conversation_id = ?6 AND role = 'assistant'
            "#,
        )
        .bind(content)
        .bind(status)
        .bind(response_id)
        .bind(error_code)
        .bind(&request.assistant_message_id)
        .bind(&request.conversation_id)
        .execute(&mut *transaction)
        .await?;

        if result.rows_affected() != 1 {
            return Err(sqlx::Error::RowNotFound);
        }

        sqlx::query("UPDATE conversations SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1")
            .bind(&request.conversation_id)
            .execute(&mut *transaction)
            .await?;

        transaction.commit().await
    }

    pub async fn list_conversations(&self) -> Result<Vec<ConversationRecord>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, created_at, updated_at FROM conversations ORDER BY updated_at DESC, id",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(ConversationRecord {
                    id: row.try_get("id")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn conversation_messages(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<MessageRecord>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, conversation_id, role, content, status, response_id, error_code,
                   created_at, updated_at
            FROM messages
            WHERE conversation_id = ?1
            ORDER BY created_at, rowid
            "#,
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(MessageRecord {
                    id: row.try_get("id")?,
                    conversation_id: row.try_get("conversation_id")?,
                    role: row.try_get("role")?,
                    content: row.try_get("content")?,
                    status: row.try_get("status")?,
                    response_id: row.try_get("response_id")?,
                    error_code: row.try_get("error_code")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ChatStreamRequest {
        ChatStreamRequest {
            conversation_id: "conversation-1".to_owned(),
            user_message_id: "user-1".to_owned(),
            assistant_message_id: "assistant-1".to_owned(),
            prompt: "Hello".to_owned(),
        }
    }

    #[tokio::test]
    async fn completed_conversation_survives_database_restart() {
        let directory = tempfile::tempdir().unwrap();
        let database_path = directory.path().join("curio.sqlite3");
        let database_url = format!("sqlite://{}", database_path.display());

        {
            let database = Database::connect(&database_url).await.unwrap();
            database.begin_chat(&request()).await.unwrap();
            database
                .complete_assistant(&request(), "Hello back", "response-1")
                .await
                .unwrap();
        }

        let database = Database::connect(&database_url).await.unwrap();
        assert_eq!(database.list_conversations().await.unwrap().len(), 1);
        let messages = database
            .conversation_messages("conversation-1")
            .await
            .unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[1].content, "Hello back");
        assert_eq!(messages[1].status, "completed");
        assert_eq!(messages[1].response_id.as_deref(), Some("response-1"));
    }

    #[tokio::test]
    async fn failed_assistant_is_stored_once_with_buffered_content() {
        let database = Database::connect("sqlite::memory:").await.unwrap();
        database.begin_chat(&request()).await.unwrap();
        database
            .fail_assistant(&request(), "partial response", "provider_error")
            .await
            .unwrap();

        let messages = database
            .conversation_messages("conversation-1")
            .await
            .unwrap();
        let assistant = &messages[1];
        assert_eq!(assistant.content, "partial response");
        assert_eq!(assistant.status, "failed");
        assert_eq!(assistant.error_code.as_deref(), Some("provider_error"));
    }
}
