use crate::{
    assistant::{model::ChatStreamRequest, repository::AssistantRepository},
    postgres_test_support::PostgresFixture,
};

fn request() -> ChatStreamRequest {
    request_for("conversation-1", "user-1", "assistant-1")
}

fn request_for(
    conversation_id: &str,
    user_message_id: &str,
    assistant_message_id: &str,
) -> ChatStreamRequest {
    ChatStreamRequest {
        conversation_id: conversation_id.to_owned(),
        user_message_id: user_message_id.to_owned(),
        assistant_message_id: assistant_message_id.to_owned(),
        prompt: "Hello".to_owned(),
    }
}

#[tokio::test]
async fn completed_conversation_survives_pool_reconnect_with_tied_ordering() {
    let Some(postgres) = PostgresFixture::provision(
        "completed_conversation_survives_pool_reconnect_with_tied_ordering",
    )
    .await
    else {
        return;
    };
    let repository = AssistantRepository::new(postgres.database().clone());
    repository.begin_chat(&request()).await.unwrap();
    repository
        .complete_assistant(&request(), "Hello back", "response-1")
        .await
        .unwrap();

    postgres.database().close().await;
    let reopened_database = postgres.reconnect().await;
    let reopened_repository = AssistantRepository::new(reopened_database.clone());

    assert_eq!(
        reopened_repository
            .list_conversations()
            .await
            .unwrap()
            .len(),
        1
    );
    let messages = reopened_repository
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
    assert_eq!(
        messages[0].created_at, messages[1].created_at,
        "both inserts share a transaction timestamp; sort_order must break the tie"
    );

    let encoded = serde_json::to_value(&messages).unwrap();
    assert_millisecond_utc(encoded[0]["createdAt"].as_str().unwrap());
    assert_millisecond_utc(encoded[1]["updatedAt"].as_str().unwrap());

    reopened_database.close().await;
    postgres.cleanup().await;
}

#[tokio::test]
async fn conversation_upsert_and_list_ordering_are_preserved() {
    let Some(postgres) = PostgresFixture::provision("conversation_upsert_and_list_ordering").await
    else {
        return;
    };
    let repository = AssistantRepository::new(postgres.database().clone());
    let first = request();
    let same_conversation = request_for("conversation-1", "user-2", "assistant-2");
    let other_conversation = request_for("conversation-2", "user-3", "assistant-3");

    repository.begin_chat(&first).await.unwrap();
    repository.begin_chat(&same_conversation).await.unwrap();
    repository.begin_chat(&other_conversation).await.unwrap();

    let conversation_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM conversations")
        .fetch_one(postgres.database().pool())
        .await
        .unwrap();
    assert_eq!(conversation_count, 2);
    assert_eq!(
        repository
            .conversation_messages("conversation-1")
            .await
            .unwrap()
            .len(),
        4
    );
    assert!(
        repository
            .conversation_messages("missing-conversation")
            .await
            .unwrap()
            .is_empty()
    );

    sqlx::query(
        "UPDATE conversations SET updated_at = \
         CASE id \
             WHEN 'conversation-1' THEN '2026-09-01T12:00:00Z'::timestamptz \
             WHEN 'conversation-2' THEN '2026-09-01T13:00:00Z'::timestamptz \
         END",
    )
    .execute(postgres.database().pool())
    .await
    .unwrap();
    let conversations = repository.list_conversations().await.unwrap();
    assert_eq!(conversations[0].id, "conversation-2");
    assert_eq!(conversations[1].id, "conversation-1");

    sqlx::query("UPDATE conversations SET updated_at = '2026-09-01T14:00:00Z'")
        .execute(postgres.database().pool())
        .await
        .unwrap();
    let conversations = repository.list_conversations().await.unwrap();
    assert_eq!(conversations[0].id, "conversation-1");
    assert_eq!(conversations[1].id, "conversation-2");

    postgres.cleanup().await;
}

#[tokio::test]
async fn failed_assistant_is_stored_once_with_buffered_content() {
    let Some(postgres) =
        PostgresFixture::provision("failed_assistant_is_stored_once_with_buffered_content").await
    else {
        return;
    };
    let repository = AssistantRepository::new(postgres.database().clone());
    repository.begin_chat(&request()).await.unwrap();
    repository
        .fail_assistant(&request(), "partial response", "provider_error")
        .await
        .unwrap();

    let messages = repository
        .conversation_messages("conversation-1")
        .await
        .unwrap();
    let assistant = &messages[1];
    assert_eq!(assistant.content, "partial response");
    assert_eq!(assistant.status, "failed");
    assert_eq!(assistant.error_code.as_deref(), Some("provider_error"));

    postgres.cleanup().await;
}

#[tokio::test]
async fn interrupted_assistant_preserves_partial_content_and_error_code() {
    let Some(postgres) =
        PostgresFixture::provision("interrupted_assistant_preserves_partial_content").await
    else {
        return;
    };
    let repository = AssistantRepository::new(postgres.database().clone());
    repository.begin_chat(&request()).await.unwrap();
    repository
        .interrupt_assistant(&request(), "partial response", "client_disconnected")
        .await
        .unwrap();

    let messages = repository
        .conversation_messages("conversation-1")
        .await
        .unwrap();
    let assistant = &messages[1];
    assert_eq!(assistant.content, "partial response");
    assert_eq!(assistant.status, "interrupted");
    assert_eq!(assistant.error_code.as_deref(), Some("client_disconnected"));

    postgres.cleanup().await;
}

#[tokio::test]
async fn begin_chat_rolls_back_every_insert_when_a_message_conflicts() {
    let Some(postgres) = PostgresFixture::provision("begin_chat_transaction_is_atomic").await
    else {
        return;
    };
    let repository = AssistantRepository::new(postgres.database().clone());
    let mut conflicting = request();
    conflicting.assistant_message_id = conflicting.user_message_id.clone();

    assert!(repository.begin_chat(&conflicting).await.is_err());
    let conversations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM conversations")
        .fetch_one(postgres.database().pool())
        .await
        .unwrap();
    let messages: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages")
        .fetch_one(postgres.database().pool())
        .await
        .unwrap();
    assert_eq!(conversations, 0);
    assert_eq!(messages, 0);

    postgres.cleanup().await;
}

#[tokio::test]
async fn finishing_requires_exactly_one_matching_assistant_row() {
    let Some(postgres) =
        PostgresFixture::provision("finish_assistant_requires_one_matching_row").await
    else {
        return;
    };
    let repository = AssistantRepository::new(postgres.database().clone());
    repository.begin_chat(&request()).await.unwrap();
    sqlx::query("DELETE FROM messages WHERE id = $1")
        .bind(&request().assistant_message_id)
        .execute(postgres.database().pool())
        .await
        .unwrap();

    assert!(matches!(
        repository
            .complete_assistant(&request(), "content", "response-1")
            .await,
        Err(sqlx::Error::RowNotFound)
    ));

    postgres.cleanup().await;
}

#[tokio::test]
async fn readiness_and_conversation_cascade_are_enforced() {
    let Some(postgres) =
        PostgresFixture::provision("readiness_and_conversation_cascade_are_enforced").await
    else {
        return;
    };
    let repository = AssistantRepository::new(postgres.database().clone());
    postgres.database().readiness().await.unwrap();
    repository.begin_chat(&request()).await.unwrap();

    sqlx::query("DELETE FROM conversations WHERE id = $1")
        .bind(&request().conversation_id)
        .execute(postgres.database().pool())
        .await
        .unwrap();
    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages")
        .fetch_one(postgres.database().pool())
        .await
        .unwrap();
    assert_eq!(remaining, 0);

    postgres.cleanup().await;
}

fn assert_millisecond_utc(timestamp: &str) {
    chrono::DateTime::parse_from_rfc3339(timestamp).expect("timestamp must be RFC 3339");
    assert_eq!(
        timestamp.len(),
        24,
        "timestamp must have millisecond precision"
    );
    assert_eq!(&timestamp[19..20], ".");
    assert_eq!(&timestamp[23..], "Z");
}
