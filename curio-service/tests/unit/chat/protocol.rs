use super::*;

fn request() -> ChatStreamRequest {
    ChatStreamRequest {
        conversation_id: "conversation-1".to_owned(),
        user_message_id: "user-1".to_owned(),
        assistant_message_id: "assistant-1".to_owned(),
        prompt: "Hello".to_owned(),
    }
}

#[test]
fn request_uses_the_public_camel_case_contract() {
    let decoded: ChatStreamRequest = serde_json::from_str(
        r#"{"conversationId":"conversation-1","userMessageId":"user-1","assistantMessageId":"assistant-1","prompt":"Hello"}"#,
    )
    .unwrap();

    assert_eq!(decoded, request());
}

#[test]
fn token_event_matches_the_exact_sse_contract() {
    let event = ChatStreamEvent::token(&request(), "Hello");

    assert_eq!(
        event.encode().unwrap(),
        "event: token\ndata: {\"conversationId\":\"conversation-1\",\"messageId\":\"assistant-1\",\"token\":\"Hello\"}\n\n"
    );
}

#[test]
fn error_event_matches_the_exact_sse_contract() {
    let event = ChatStreamEvent::error(&request(), "provider_error", "Request failed");

    assert_eq!(
        event.encode().unwrap(),
        "event: error\ndata: {\"conversationId\":\"conversation-1\",\"messageId\":\"assistant-1\",\"code\":\"provider_error\",\"message\":\"Request failed\"}\n\n"
    );
}

#[test]
fn done_event_matches_the_exact_sse_contract() {
    let event = ChatStreamEvent::done(&request(), "response-1");

    assert_eq!(
        event.encode().unwrap(),
        "event: done\ndata: {\"conversationId\":\"conversation-1\",\"messageId\":\"assistant-1\",\"responseId\":\"response-1\"}\n\n"
    );
}
