use futures_util::StreamExt;
use reqwest::Client;
use serde::Serialize;
use tokio::sync::mpsc;

const PROVIDER_ERROR_MESSAGE: &str = "The model provider could not complete the response.";
const PROVIDER_STREAM_ERROR_MESSAGE: &str = "The model provider returned an invalid stream.";
const PROVIDER_UNAVAILABLE_MESSAGE: &str = "The model provider is temporarily unavailable.";

#[derive(Clone)]
pub struct OpenAiClient {
    http: Client,
    api_key: String,
    model: String,
    base_url: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OpenAiEvent {
    Token(String),
    Done(String),
    Error {
        code: &'static str,
        message: &'static str,
    },
}

#[derive(Serialize)]
struct ResponsesRequest<'a> {
    model: &'a str,
    input: &'a str,
    stream: bool,
}

impl OpenAiClient {
    pub fn new(api_key: String, model: String, base_url: String) -> Self {
        Self {
            http: Client::new(),
            api_key,
            model,
            base_url: base_url.trim_end_matches('/').to_owned(),
        }
    }

    pub fn stream(&self, prompt: String) -> mpsc::Receiver<OpenAiEvent> {
        let (sender, receiver) = mpsc::channel(32);
        let client = self.clone();

        tokio::spawn(async move {
            client.run_stream(prompt, sender).await;
        });

        receiver
    }

    async fn run_stream(&self, prompt: String, sender: mpsc::Sender<OpenAiEvent>) {
        let response = self
            .http
            .post(format!("{}/v1/responses", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&ResponsesRequest {
                model: &self.model,
                input: &prompt,
                stream: true,
            })
            .send()
            .await;

        let Ok(response) = response else {
            send_error(
                &sender,
                "provider_unavailable",
                PROVIDER_UNAVAILABLE_MESSAGE,
            )
            .await;
            return;
        };

        if !response.status().is_success() {
            send_error(&sender, "provider_error", PROVIDER_ERROR_MESSAGE).await;
            return;
        }

        let mut parser = OpenAiSseParser::default();
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let Ok(chunk) = chunk else {
                send_error(
                    &sender,
                    "provider_unavailable",
                    PROVIDER_UNAVAILABLE_MESSAGE,
                )
                .await;
                return;
            };

            match parser.push(&chunk) {
                Ok(events) => {
                    if forward_events(&sender, events).await {
                        return;
                    }
                }
                Err(()) => {
                    send_error(
                        &sender,
                        "malformed_provider_stream",
                        PROVIDER_STREAM_ERROR_MESSAGE,
                    )
                    .await;
                    return;
                }
            }
        }

        match parser.finish() {
            Ok(events) => {
                forward_events(&sender, events).await;
            }
            Err(()) => {
                send_error(
                    &sender,
                    "incomplete_provider_stream",
                    PROVIDER_STREAM_ERROR_MESSAGE,
                )
                .await;
            }
        }
    }
}

async fn forward_events(sender: &mpsc::Sender<OpenAiEvent>, events: Vec<OpenAiEvent>) -> bool {
    for event in events {
        let terminal = matches!(event, OpenAiEvent::Done(_) | OpenAiEvent::Error { .. });
        if sender.send(event).await.is_err() || terminal {
            return true;
        }
    }

    false
}

async fn send_error(sender: &mpsc::Sender<OpenAiEvent>, code: &'static str, message: &'static str) {
    let _ = sender.send(OpenAiEvent::Error { code, message }).await;
}

#[derive(Default)]
struct OpenAiSseParser {
    buffer: Vec<u8>,
    terminal_event_received: bool,
}

impl OpenAiSseParser {
    fn push(&mut self, chunk: &[u8]) -> Result<Vec<OpenAiEvent>, ()> {
        if self.terminal_event_received && chunk.iter().any(|byte| !byte.is_ascii_whitespace()) {
            return Err(());
        }

        self.buffer.extend_from_slice(chunk);
        let mut events = Vec::new();

        while let Some((boundary_index, boundary_length)) = find_event_boundary(&self.buffer) {
            let event_block = self.buffer[..boundary_index].to_vec();
            self.buffer.drain(..boundary_index + boundary_length);
            if let Some(event) = parse_provider_event(&event_block)? {
                self.accept(event, &mut events)?;
            }
        }

        Ok(events)
    }

    fn finish(&mut self) -> Result<Vec<OpenAiEvent>, ()> {
        let mut events = Vec::new();
        if self.buffer.iter().any(|byte| !byte.is_ascii_whitespace()) {
            let event_block = std::mem::take(&mut self.buffer);
            if let Some(event) = parse_provider_event(&event_block)? {
                self.accept(event, &mut events)?;
            }
        }

        if !self.terminal_event_received {
            return Err(());
        }

        Ok(events)
    }

    fn accept(&mut self, event: OpenAiEvent, events: &mut Vec<OpenAiEvent>) -> Result<(), ()> {
        if self.terminal_event_received {
            return Err(());
        }

        if matches!(event, OpenAiEvent::Done(_) | OpenAiEvent::Error { .. }) {
            self.terminal_event_received = true;
        }
        events.push(event);
        Ok(())
    }
}

fn find_event_boundary(buffer: &[u8]) -> Option<(usize, usize)> {
    for index in 0..buffer.len().saturating_sub(1) {
        if buffer[index..].starts_with(b"\n\n") {
            return Some((index, 2));
        }
        if buffer[index..].starts_with(b"\r\n\r\n") {
            return Some((index, 4));
        }
    }

    None
}

fn parse_provider_event(event_block: &[u8]) -> Result<Option<OpenAiEvent>, ()> {
    let event_block = std::str::from_utf8(event_block).map_err(|_| ())?;
    let mut data_lines = Vec::new();

    for line in event_block.lines() {
        if let Some(data) = line.strip_prefix("data:") {
            data_lines.push(data.strip_prefix(' ').unwrap_or(data));
        }
    }

    if data_lines.is_empty() {
        return Ok(None);
    }

    let data = data_lines.join("\n");
    if data == "[DONE]" {
        return Ok(None);
    }

    let payload: serde_json::Value = serde_json::from_str(&data).map_err(|_| ())?;
    let event_type = payload
        .get("type")
        .and_then(|value| value.as_str())
        .ok_or(())?;

    match event_type {
        "response.output_text.delta" => payload
            .get("delta")
            .and_then(|value| value.as_str())
            .map(|delta| Some(OpenAiEvent::Token(delta.to_owned())))
            .ok_or(()),
        "response.completed" => payload
            .get("response")
            .and_then(|response| response.get("id"))
            .and_then(|value| value.as_str())
            .map(|id| Some(OpenAiEvent::Done(id.to_owned())))
            .ok_or(()),
        "response.failed" | "response.incomplete" | "error" => Ok(Some(OpenAiEvent::Error {
            code: "provider_error",
            message: PROVIDER_ERROR_MESSAGE,
        })),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
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
}
