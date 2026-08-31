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
