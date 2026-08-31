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
                message: PROVIDER_ERROR_MESSAGE,
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
