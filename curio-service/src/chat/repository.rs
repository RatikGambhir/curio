use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, Postgres, QueryBuilder};

use crate::{chat::protocol::ChatStreamRequest, database::Database};

#[derive(Clone)]
pub struct ChatRepository {
    database: Database,
}

#[derive(Debug, FromRow, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConversationRecord {
    pub id: String,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MessageRecord {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub status: String,
    pub response_id: Option<String>,
    pub error_code: Option<String>,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "crate::database::serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

impl ChatRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn begin_chat(&self, request: &ChatStreamRequest) -> Result<(), sqlx::Error> {
        let mut transaction = self.database.pool().begin().await?;

        let mut conversation =
            QueryBuilder::<Postgres>::new("INSERT INTO conversations (id) VALUES (");
        conversation
            .push_bind(&request.conversation_id)
            .push(") ON CONFLICT (id) DO UPDATE SET updated_at = CURRENT_TIMESTAMP");
        conversation.build().execute(&mut *transaction).await?;

        let mut user_message = QueryBuilder::<Postgres>::new(
            "INSERT INTO messages (id, conversation_id, role, content, status) VALUES (",
        );
        user_message
            .push_bind(&request.user_message_id)
            .push(", ")
            .push_bind(&request.conversation_id)
            .push(", 'user', ")
            .push_bind(&request.prompt)
            .push(", 'completed')");
        user_message.build().execute(&mut *transaction).await?;

        let mut assistant_message = QueryBuilder::<Postgres>::new(
            "INSERT INTO messages (id, conversation_id, role, content, status) VALUES (",
        );
        assistant_message
            .push_bind(&request.assistant_message_id)
            .push(", ")
            .push_bind(&request.conversation_id)
            .push(", 'assistant', '', 'pending')");
        assistant_message.build().execute(&mut *transaction).await?;

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
        let mut transaction = self.database.pool().begin().await?;
        let mut message = QueryBuilder::<Postgres>::new("UPDATE messages SET content = ");
        message
            .push_bind(content)
            .push(", status = ")
            .push_bind(status)
            .push(", response_id = ")
            .push_bind(response_id)
            .push(", error_code = ")
            .push_bind(error_code)
            .push(", updated_at = CURRENT_TIMESTAMP WHERE id = ")
            .push_bind(&request.assistant_message_id)
            .push(" AND conversation_id = ")
            .push_bind(&request.conversation_id)
            .push(" AND role = 'assistant'");
        let result = message.build().execute(&mut *transaction).await?;

        if result.rows_affected() != 1 {
            return Err(sqlx::Error::RowNotFound);
        }

        let mut conversation = QueryBuilder::<Postgres>::new(
            "UPDATE conversations SET updated_at = CURRENT_TIMESTAMP WHERE id = ",
        );
        conversation.push_bind(&request.conversation_id);
        conversation.build().execute(&mut *transaction).await?;

        transaction.commit().await
    }

    pub async fn list_conversations(&self) -> Result<Vec<ConversationRecord>, sqlx::Error> {
        QueryBuilder::<Postgres>::new(
            "SELECT id, created_at, updated_at FROM conversations ORDER BY updated_at DESC, id",
        )
        .build_query_as::<ConversationRecord>()
        .fetch_all(self.database.pool())
        .await
    }

    pub async fn conversation_messages(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<MessageRecord>, sqlx::Error> {
        let mut messages = QueryBuilder::<Postgres>::new(
            "SELECT id, conversation_id, role, content, status, response_id, error_code, \
             created_at, updated_at FROM messages WHERE conversation_id = ",
        );
        messages
            .push_bind(conversation_id)
            .push(" ORDER BY created_at, sort_order");
        messages
            .build_query_as::<MessageRecord>()
            .fetch_all(self.database.pool())
            .await
    }
}
