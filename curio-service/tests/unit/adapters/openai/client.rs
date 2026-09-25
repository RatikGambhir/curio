use super::*;

fn event(data: &str) -> String {
    format!("data: {data}\n\n")
}

#[test]
fn parses_split_chunks_and_multiple_events() {
    let mut parser = OpenAiSseParser::default();
    let stream = format!(
        "{}{}",
        event(r#"{"type":"response.output_text.delta","delta":"Hel"}"#),
        event(r#"{"type":"response.output_text.delta","delta":"lo"}"#)
    );

    assert_eq!(parser.push(&stream.as_bytes()[..17]).unwrap(), Vec::new());
    assert_eq!(
        parser.push(&stream.as_bytes()[17..]).unwrap(),
        vec![
            OpenAiEvent::Token("Hel".to_owned()),
            OpenAiEvent::Token("lo".to_owned())
        ]
    );
}

#[test]
fn normalizes_completion() {
    let mut parser = OpenAiSseParser::default();
    let stream = event(r#"{"type":"response.completed","response":{"id":"response-1"}}"#);

    assert_eq!(
        parser.push(stream.as_bytes()).unwrap(),
        vec![OpenAiEvent::Done("response-1".to_owned())]
    );
    assert!(parser.finish().unwrap().is_empty());
}

#[test]
fn normalizes_all_provider_failure_events() {
    for event_type in ["response.failed", "response.incomplete", "error"] {
        let mut parser = OpenAiSseParser::default();
        let stream = event(&format!(r#"{{"type":"{event_type}"}}"#));

        assert_eq!(
            parser.push(stream.as_bytes()).unwrap(),
            vec![OpenAiEvent::Error {
                code: "provider_error",
                message: PROVIDER_ERROR_MESSAGE.to_owned(),
            }]
        );
    }
}

#[test]
fn rejects_malformed_and_truncated_streams() {
    let mut malformed = OpenAiSseParser::default();
    assert!(malformed.push(b"data: not-json\n\n").is_err());

    let mut truncated = OpenAiSseParser::default();
    let token = event(r#"{"type":"response.output_text.delta","delta":"partial"}"#);
    assert!(truncated.push(token.as_bytes()).is_ok());
    assert!(truncated.finish().is_err());
}

#[test]
fn forwards_the_provider_message_from_a_failed_stream() {
    let mut parser = OpenAiSseParser::default();
    let stream = event(
        r#"{"type":"response.failed","response":{"error":{"code":"server_error","message":"The model produced no output."}}}"#,
    );

    assert_eq!(
        parser.push(stream.as_bytes()).unwrap(),
        vec![OpenAiEvent::Error {
            code: "provider_error",
            message: "OpenAI stream failed (server_error): The model produced no output."
                .to_owned(),
        }]
    );
}

#[test]
fn forwards_the_reason_from_an_incomplete_stream() {
    let mut parser = OpenAiSseParser::default();
    let stream = event(
        r#"{"type":"response.incomplete","response":{"incomplete_details":{"reason":"max_output_tokens"}}}"#,
    );

    assert_eq!(
        parser.push(stream.as_bytes()).unwrap(),
        vec![OpenAiEvent::Error {
            code: "provider_error",
            message: "OpenAI stream ended early: max_output_tokens".to_owned(),
        }]
    );
}

#[test]
fn formats_status_errors_with_the_provider_body() {
    let body = r#"{"error":{"message":"Rate limit reached for gpt-4o.","type":"requests","code":"rate_limit_exceeded"}}"#;
    let parsed = parse_provider_error_body(body);

    assert_eq!(
        format_status_error(reqwest::StatusCode::TOO_MANY_REQUESTS, &parsed),
        "OpenAI returned HTTP 429 (rate_limit_exceeded): Rate limit reached for gpt-4o."
    );
    assert_eq!(
        provider_error_code_for_status(reqwest::StatusCode::TOO_MANY_REQUESTS),
        "provider_rate_limited"
    );
    assert_eq!(
        provider_error_code_for_status(reqwest::StatusCode::UNAUTHORIZED),
        "provider_auth_error"
    );
    assert_eq!(
        provider_error_code_for_status(reqwest::StatusCode::BAD_GATEWAY),
        "provider_unavailable"
    );
    assert_eq!(
        provider_error_code_for_status(reqwest::StatusCode::BAD_REQUEST),
        "provider_request_invalid"
    );
}

#[test]
fn falls_back_when_the_body_is_not_json() {
    let parsed = parse_provider_error_body("  upstream connect error  ");
    assert_eq!(parsed.message.as_deref(), Some("upstream connect error"));

    let empty = parse_provider_error_body("");
    assert_eq!(empty.message, None);
    assert_eq!(
        format_status_error(reqwest::StatusCode::INTERNAL_SERVER_ERROR, &empty),
        format!("OpenAI returned HTTP 500: {PROVIDER_ERROR_MESSAGE}")
    );
}

#[test]
fn redacts_credentials_quoted_back_by_the_provider() {
    assert_eq!(
        redact_secrets("Incorrect API key provided: sk-proj-AbC123_x-y.z. Check your key."),
        "Incorrect API key provided: sk-***. Check your key."
    );
    // The Bearer marker matches first and swallows the whole token value.
    assert_eq!(
        redact_secrets("header Bearer sk-abc123 rejected"),
        "header Bearer *** rejected"
    );
    assert_eq!(redact_secrets("no secrets here"), "no secrets here");
}

#[test]
fn truncates_oversized_provider_messages() {
    let long = "é".repeat(400);
    let truncated = truncate_message(long);

    assert!(truncated.len() <= 503, "length was {}", truncated.len());
    assert!(truncated.ends_with('…'));
    assert!(truncate_message("short".to_owned()) == "short");
}

#[tokio::test]
async fn forwards_deltas_and_resolves_the_terminal_result() {
    let (sender, mut receiver) = mpsc::channel(4);
    let result = forward_events(
        &sender,
        vec![
            OpenAiEvent::Token("Hel".to_owned()),
            OpenAiEvent::Token("lo".to_owned()),
            OpenAiEvent::Done("response-1".to_owned()),
        ],
    )
    .await;

    assert_eq!(result, Some(Ok("response-1".to_owned())));
    assert_eq!(receiver.recv().await.as_deref(), Some("Hel"));
    assert_eq!(receiver.recv().await.as_deref(), Some("lo"));
    assert_eq!(forward_events(&sender, Vec::new()).await, None);
}

#[tokio::test]
async fn provider_error_events_become_sanitized_stream_errors() {
    let (sender, _receiver) = mpsc::channel(1);
    let result = forward_events(
        &sender,
        vec![OpenAiEvent::Error {
            code: "provider_error",
            message: "Unavailable".to_owned(),
        }],
    )
    .await;

    assert_eq!(
        result,
        Some(Err(ChatStreamError::new("provider_error", "Unavailable")))
    );
}
