use crate::{
    assistant::model::{ChatStreamRequest, ConversationRecord, MessageRecord},
    core::sql::Sql,
    database::Database,
};

#[derive(Clone)]
pub struct AssistantRepository {
    database: Database,
}

impl AssistantRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn begin_chat(&self, request: &ChatStreamRequest) -> Result<(), sqlx::Error> {
        let mut transaction = self.database.pool().begin().await?;

        Sql::insert_into("conversations")
            .set("id", &request.conversation_id)
            .upsert_on("id")
            .execute(&mut *transaction)
            .await?;

        Sql::insert_into("messages")
            .set("id", &request.user_message_id)
            .set("conversation_id", &request.conversation_id)
            .set_literal("role", "'user'")
            .set("content", &request.prompt)
            .set_literal("status", "'completed'")
            .execute(&mut *transaction)
            .await?;

        Sql::insert_into("messages")
            .set("id", &request.assistant_message_id)
            .set("conversation_id", &request.conversation_id)
            .set_literal("role", "'assistant'")
            .set_literal("content", "''")
            .set_literal("status", "'pending'")
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
        let mut transaction = self.database.pool().begin().await?;

        Sql::update("messages")
            .set("content", content)
            .set("status", status)
            .set("response_id", response_id)
            .set("error_code", error_code)
            .set_now("updated_at")
            .filter("id = ?", &request.assistant_message_id)
            .filter("conversation_id = ?", &request.conversation_id)
            .filter_literal("role = 'assistant'")
            .execute_one(&mut *transaction)
            .await?;

        Sql::update("conversations")
            .set_now("updated_at")
            .filter("id = ?", &request.conversation_id)
            .execute(&mut *transaction)
            .await?;

        transaction.commit().await
    }

    pub async fn list_conversations(&self) -> Result<Vec<ConversationRecord>, sqlx::Error> {
        Sql::select(ConversationRecord::COLUMNS)
            .from("conversations")
            .order_by("updated_at DESC, id")
            .fetch_all(self.database.pool())
            .await
    }

    pub async fn conversation_messages(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<MessageRecord>, sqlx::Error> {
        Sql::select(MessageRecord::COLUMNS)
            .from("messages")
            .filter("conversation_id = ?", conversation_id)
            .order_by("created_at, sort_order")
            .fetch_all(self.database.pool())
            .await
    }
}

#[cfg(test)]
#[path = "../../tests/unit/assistant/repository.rs"]
mod tests;
