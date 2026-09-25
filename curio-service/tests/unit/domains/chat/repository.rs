use crate::{
    domains::chat::{
        model::{AssistantOutcome, ChatStreamRequest},
        repository::ChatRepository,
        service::ChatStore,
    },
    postgres_test_support::PostgresFixture,
};

fn request() -> ChatStreamRequest {
    ChatStreamRequest {
        conversation_id: "conversation-1".to_owned(),
        user_message_id: "user-1".to_owned(),
        assistant_message_id: "assistant-1".to_owned(),
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
    let repository = ChatRepository::new(postgres.database().clone());
    repository.start_run(&request()).await.unwrap();
    repository
        .finish_run(
            &request(),
            "Hello back",
            AssistantOutcome::Completed {
                response_id: "response-1",
            },
        )
        .await
        .unwrap();

    postgres.database().close().await;
    let reopened_database = postgres.reconnect().await;
    let reopened_repository = ChatRepository::new(reopened_database.clone());

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
async fn failed_assistant_is_stored_once_with_buffered_content() {
    let Some(postgres) =
        PostgresFixture::provision("failed_assistant_is_stored_once_with_buffered_content").await
    else {
        return;
    };
    let repository = ChatRepository::new(postgres.database().clone());
    repository.start_run(&request()).await.unwrap();
    repository
        .finish_run(
            &request(),
            "partial response",
            AssistantOutcome::Failed {
                code: "provider_error",
            },
        )
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
async fn readiness_and_conversation_cascade_are_enforced() {
    let Some(postgres) =
        PostgresFixture::provision("readiness_and_conversation_cascade_are_enforced").await
    else {
        return;
    };
    let repository = ChatRepository::new(postgres.database().clone());
    postgres.database().readiness().await.unwrap();
    repository.start_run(&request()).await.unwrap();

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

#[tokio::test]
async fn duplicate_message_rolls_back_the_entire_new_conversation() {
    let Some(postgres) = PostgresFixture::provision("chat_begin_rollback").await else {
        return;
    };
    let repository = ChatRepository::new(postgres.database().clone());
    repository.start_run(&request()).await.unwrap();
    let mut conflicting = request();
    conflicting.conversation_id = "must-not-persist".into();
    conflicting.assistant_message_id = "new-assistant".into();
    assert!(repository.start_run(&conflicting).await.is_err());
    assert_eq!(repository.list_conversations().await.unwrap().len(), 1);
    assert!(
        repository
            .conversation_messages("must-not-persist")
            .await
            .unwrap()
            .is_empty()
    );
    postgres.cleanup().await;
}

#[tokio::test]
async fn assistant_update_requires_matching_message_and_conversation() {
    let Some(postgres) = PostgresFixture::provision("chat_finish_scope").await else {
        return;
    };
    let repository = ChatRepository::new(postgres.database().clone());
    repository.start_run(&request()).await.unwrap();
    let mut mismatched = request();
    mismatched.conversation_id = "different-conversation".into();
    assert!(
        repository
            .finish_run(
                &mismatched,
                "incorrect",
                AssistantOutcome::Completed {
                    response_id: "incorrect"
                }
            )
            .await
            .is_err()
    );
    let messages = repository
        .conversation_messages("conversation-1")
        .await
        .unwrap();
    assert_eq!(messages[1].status, "pending");
    assert_eq!(messages[1].content, "");
    assert_eq!(messages[1].response_id, None);
    postgres.cleanup().await;
}
