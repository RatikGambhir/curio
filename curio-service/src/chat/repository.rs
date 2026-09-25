//! PostgreSQL conversation persistence; each lifecycle mutation is atomic.
use chrono::{DateTime, Utc};
use sqlx::FromRow;

use super::{
    domain::{AssistantOutcome, ChatStorageError, ChatTurn, ConversationRecord, MessageRecord},
    service::ChatStore,
};
use crate::{
    database::Database,
    query::{
        Assignment, Comparison, Direction, Expression, FieldWriter, InsertQuery, SelectQuery,
        SqlColumn, SqlField, UpdateQuery,
    },
};

#[derive(Clone)]
pub struct PostgresChatRepository {
    database: Database,
}

impl PostgresChatRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl ChatStore for PostgresChatRepository {
    async fn begin_chat(&self, request: &ChatTurn) -> Result<(), ChatStorageError> {
        self.begin_transaction(request)
            .await
            .map_err(|error| storage_error("chat_begin", error))
    }

    async fn finish_assistant(
        &self,
        request: &ChatTurn,
        content: &str,
        outcome: AssistantOutcome<'_>,
    ) -> Result<(), ChatStorageError> {
        self.finish_transaction(request, content, outcome)
            .await
            .map_err(|error| storage_error("chat_finish_assistant", error))
    }

    async fn list_conversations(&self) -> Result<Vec<ConversationRecord>, ChatStorageError> {
        SelectQuery::new(
            "conversations",
            [
                ConversationColumn::Id,
                ConversationColumn::CreatedAt,
                ConversationColumn::UpdatedAt,
            ],
        )
        .order_by(ConversationColumn::UpdatedAt, Direction::Descending)
        .order_by(ConversationColumn::Id, Direction::Ascending)
        .build()
        .map_err(|error| storage_error("chat_query", error))?
        .build_query_as::<ConversationRow>()
        .fetch_all(self.database.pool())
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(|error| storage_error("chat_list_conversations", error))
    }

    async fn conversation_messages(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<MessageRecord>, ChatStorageError> {
        SelectQuery::new(
            "messages",
            [
                MessageColumn::Id,
                MessageColumn::ConversationId,
                MessageColumn::Role,
                MessageColumn::Content,
                MessageColumn::Status,
                MessageColumn::ResponseId,
                MessageColumn::ErrorCode,
                MessageColumn::CreatedAt,
                MessageColumn::UpdatedAt,
            ],
        )
        .filter(
            MessageColumn::ConversationId,
            Comparison::Equal,
            conversation_id,
        )
        .order_by(MessageColumn::CreatedAt, Direction::Ascending)
        .order_by(MessageColumn::SortOrder, Direction::Ascending)
        .build()
        .map_err(|error| storage_error("chat_query", error))?
        .build_query_as::<MessageRow>()
        .fetch_all(self.database.pool())
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(|error| storage_error("chat_conversation_messages", error))
    }
}

impl PostgresChatRepository {
    async fn begin_transaction(&self, request: &ChatTurn) -> Result<(), sqlx::Error> {
        let mut transaction = self.database.pool().begin().await?;
        InsertQuery::new("conversations")
            .value(ConversationField::Id(&request.conversation_id))
            .on_conflict(
                ConversationColumn::Id,
                [Assignment::CurrentTimestamp(ConversationColumn::UpdatedAt)],
            )
            .build()?
            .build()
            .execute(&mut *transaction)
            .await?;
        InsertQuery::new("messages")
            .value(MessageField::Id(&request.user_message_id))
            .value(MessageField::ConversationId(&request.conversation_id))
            .value(MessageField::Role("user"))
            .value(MessageField::Content(&request.prompt))
            .value(MessageField::Status("completed"))
            .build()?
            .build()
            .execute(&mut *transaction)
            .await?;
        InsertQuery::new("messages")
            .value(MessageField::Id(&request.assistant_message_id))
            .value(MessageField::ConversationId(&request.conversation_id))
            .value(MessageField::Role("assistant"))
            .value(MessageField::Content(""))
            .value(MessageField::Status("pending"))
            .build()?
            .build()
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await
    }

    async fn finish_transaction(
        &self,
        request: &ChatTurn,
        content: &str,
        outcome: AssistantOutcome<'_>,
    ) -> Result<(), sqlx::Error> {
        let (status, response_id, error_code) = match outcome {
            AssistantOutcome::Completed { response_id } => ("completed", Some(response_id), None),
            AssistantOutcome::Failed { code } => ("failed", None, Some(code)),
            AssistantOutcome::Interrupted { code } => ("interrupted", None, Some(code)),
        };
        let mut transaction = self.database.pool().begin().await?;
        let result = UpdateQuery::new("messages")
            .value(MessageField::Content(content))
            .value(MessageField::Status(status))
            .value(MessageField::ResponseId(response_id))
            .value(MessageField::ErrorCode(error_code))
            .value(MessageField::UpdatedAt(Expression::CurrentTimestamp))
            .filter(
                MessageColumn::Id,
                Comparison::Equal,
                &request.assistant_message_id,
            )
            .filter(
                MessageColumn::ConversationId,
                Comparison::Equal,
                &request.conversation_id,
            )
            .filter(MessageColumn::Role, Comparison::Equal, "assistant")
            .build()?
            .build()
            .execute(&mut *transaction)
            .await?;
        if result.rows_affected() != 1 {
            return Err(sqlx::Error::RowNotFound);
        }
        UpdateQuery::new("conversations")
            .value(ConversationField::UpdatedAt(Expression::CurrentTimestamp))
            .filter(
                ConversationColumn::Id,
                Comparison::Equal,
                &request.conversation_id,
            )
            .build()?
            .build()
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await
    }
}

fn storage_error(operation: &'static str, error: sqlx::Error) -> ChatStorageError {
    crate::diagnostics::log_database_error(operation, &error);
    ChatStorageError
}

#[derive(Clone, Copy)]
enum ConversationColumn {
    Id,
    CreatedAt,
    UpdatedAt,
}
impl SqlColumn for ConversationColumn {
    fn sql(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }
}
#[derive(Clone, Copy)]
enum MessageColumn {
    Id,
    ConversationId,
    Role,
    Content,
    Status,
    ResponseId,
    ErrorCode,
    UpdatedAt,
    CreatedAt,
    SortOrder,
}
impl SqlColumn for MessageColumn {
    fn sql(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::ConversationId => "conversation_id",
            Self::Role => "role",
            Self::Content => "content",
            Self::Status => "status",
            Self::ResponseId => "response_id",
            Self::ErrorCode => "error_code",
            Self::UpdatedAt => "updated_at",
            Self::CreatedAt => "created_at",
            Self::SortOrder => "sort_order",
        }
    }
}

#[derive(Debug, FromRow, PartialEq, Eq)]
struct ConversationRow {
    id: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, PartialEq, Eq)]
struct MessageRow {
    id: String,
    conversation_id: String,
    role: String,
    content: String,
    status: String,
    response_id: Option<String>,
    error_code: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ConversationRow> for ConversationRecord {
    fn from(row: ConversationRow) -> Self {
        Self {
            id: row.id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
impl From<MessageRow> for MessageRecord {
    fn from(row: MessageRow) -> Self {
        Self {
            id: row.id,
            conversation_id: row.conversation_id,
            role: row.role,
            content: row.content,
            status: row.status,
            response_id: row.response_id,
            error_code: row.error_code,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Typed write fields: a variant owns its column mapping and payload type.
enum MessageField<'a> {
    Id(&'a str),
    ConversationId(&'a str),
    Role(&'a str),
    Content(&'a str),
    Status(&'a str),
    ResponseId(Option<&'a str>),
    ErrorCode(Option<&'a str>),
    UpdatedAt(Expression),
}
impl SqlField for MessageField<'_> {
    type Column = MessageColumn;
    fn write(self, writer: &mut impl FieldWriter<MessageColumn>) {
        match self {
            Self::Id(value) => writer.bind(MessageColumn::Id, value),
            Self::ConversationId(value) => writer.bind(MessageColumn::ConversationId, value),
            Self::Role(value) => writer.bind(MessageColumn::Role, value),
            Self::Content(value) => writer.bind(MessageColumn::Content, value),
            Self::Status(value) => writer.bind(MessageColumn::Status, value),
            Self::ResponseId(value) => writer.bind(MessageColumn::ResponseId, value),
            Self::ErrorCode(value) => writer.bind(MessageColumn::ErrorCode, value),
            Self::UpdatedAt(value) => writer.expression(MessageColumn::UpdatedAt, value),
        }
    }
}

/// Typed write fields: a variant owns its column mapping and payload type.
enum ConversationField<'a> {
    Id(&'a str),
    UpdatedAt(Expression),
}
impl SqlField for ConversationField<'_> {
    type Column = ConversationColumn;
    fn write(self, writer: &mut impl FieldWriter<ConversationColumn>) {
        match self {
            Self::Id(value) => writer.bind(ConversationColumn::Id, value),
            Self::UpdatedAt(value) => writer.expression(ConversationColumn::UpdatedAt, value),
        }
    }
}
