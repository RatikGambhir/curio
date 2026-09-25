use futures_util::StreamExt;
use reqwest::Client;
use serde::Serialize;
use tokio::sync::mpsc;
use tracing::{Instrument, Span, error, info_span};

use super::{domain::ModelEvent as OpenAiEvent, service::ModelProvider};

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
        let provider_span = info_span!(
            parent: Span::current(),
            "openai_stream",
            model = %client.model,
            provider_request_id = tracing::field::Empty,
        );

        tokio::spawn(
            async move {
                client.run_stream(prompt, sender).await;
            }
            .instrument(provider_span),
        );

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

        let response = match response {
            Ok(response) => response,
            Err(provider_error) => {
                error!(
                    error_kind = reqwest_error_kind(&provider_error),
                    is_timeout = provider_error.is_timeout(),
                    is_connect = provider_error.is_connect(),
                    "OpenAI request failed before a response was received"
                );
                send_error(
                    &sender,
                    "provider_unavailable",
                    provider_transport_message(&provider_error),
                )
                .await;
                return;
            }
        };
        let provider_request_id = provider_request_id(&response);
        if let Some(provider_request_id) = provider_request_id.as_deref() {
            Span::current().record("provider_request_id", provider_request_id);
        }

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let provider_error = parse_provider_error_body(&body);
            error!(
                status = status.as_u16(),
                provider_error_type = provider_error.kind.as_deref().unwrap_or("unknown"),
                provider_error_code = provider_error.code.as_deref().unwrap_or("unknown"),
                "OpenAI returned a non-success HTTP status"
            );
            send_error(
                &sender,
                provider_error_code_for_status(status),
                format_status_error(status, &provider_error),
            )
            .await;
            return;
        }

        let mut parser = OpenAiSseParser::default();
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(provider_error) => {
                    error!(
                        error_kind = reqwest_error_kind(&provider_error),
                        is_timeout = provider_error.is_timeout(),
                        is_connect = provider_error.is_connect(),
                        "OpenAI response stream failed"
                    );
                    send_error(
                        &sender,
                        "provider_unavailable",
                        provider_transport_message(&provider_error),
                    )
                    .await;
                    return;
                }
            };

            match parser.push(&chunk) {
                Ok(events) => {
                    log_provider_terminal_errors(&events);
                    if forward_events(&sender, events).await {
                        return;
                    }
                }
                Err(()) => {
                    error!("OpenAI returned a malformed SSE stream");
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
                log_provider_terminal_errors(&events);
                forward_events(&sender, events).await;
            }
            Err(()) => {
                error!("OpenAI response stream ended before a valid terminal event");
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

fn provider_request_id(response: &reqwest::Response) -> Option<String> {
    response
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn reqwest_error_kind(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "timeout"
    } else if error.is_connect() {
        "connect"
    } else if error.is_builder() {
        "builder"
    } else if error.is_redirect() {
        "redirect"
    } else if error.is_body() {
        "body"
    } else if error.is_decode() {
        "decode"
    } else if error.is_request() {
        "request"
    } else {
        "unknown"
    }
}

fn log_provider_terminal_errors(events: &[OpenAiEvent]) {
    for event in events {
        if let OpenAiEvent::Error { code, message } = event {
            error!(
                provider_error_code = code,
                provider_error_message = message.as_str(),
                "OpenAI returned a terminal error event"
            );
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

async fn send_error(
    sender: &mpsc::Sender<OpenAiEvent>,
    code: &'static str,
    message: impl Into<String>,
) {
    let _ = sender
        .send(OpenAiEvent::Error {
            code,
            message: message.into(),
        })
        .await;
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
            message: stream_error_message(&payload),
        })),
        _ => Ok(None),
    }
}

/// The provider's own error description, split into the parts OpenAI sends.
#[derive(Default)]
struct ProviderErrorBody {
    message: Option<String>,
    kind: Option<String>,
    code: Option<String>,
}

fn parse_provider_error_body(body: &str) -> ProviderErrorBody {
    let Ok(payload) = serde_json::from_str::<serde_json::Value>(body) else {
        // Gateways in front of the API answer with plain text rather than JSON.
        let trimmed = body.trim();
        return ProviderErrorBody {
            message: (!trimmed.is_empty()).then(|| trimmed.to_owned()),
            ..ProviderErrorBody::default()
        };
    };

    let error = payload.get("error").unwrap_or(&payload);
    ProviderErrorBody {
        message: string_field(error, "message"),
        kind: string_field(error, "type"),
        code: string_field(error, "code"),
    }
}

fn string_field(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|field| field.as_str())
        .filter(|field| !field.is_empty())
        .map(str::to_owned)
}

/// Stable machine-readable codes. The provider's own label travels in the
/// message so the stored `error_code` keeps a bounded set of values.
fn provider_error_code_for_status(status: reqwest::StatusCode) -> &'static str {
    match status.as_u16() {
        401 | 403 => "provider_auth_error",
        429 => "provider_rate_limited",
        408 => "provider_unavailable",
        500..=599 => "provider_unavailable",
        400..=499 => "provider_request_invalid",
        _ => "provider_error",
    }
}

fn format_status_error(status: reqwest::StatusCode, error: &ProviderErrorBody) -> String {
    let mut message = format!("OpenAI returned HTTP {}", status.as_u16());
    if let Some(label) = error.code.as_deref().or(error.kind.as_deref()) {
        message.push_str(" (");
        message.push_str(label);
        message.push(')');
    }
    message.push_str(": ");
    message.push_str(error.message.as_deref().unwrap_or(PROVIDER_ERROR_MESSAGE));
    sanitize_provider_message(message)
}

fn stream_error_message(payload: &serde_json::Value) -> String {
    let candidates = [
        payload.get("error"),
        payload.get("response").and_then(|value| value.get("error")),
    ];

    for candidate in candidates.into_iter().flatten() {
        if let Some(detail) = string_field(candidate, "message") {
            let label = string_field(candidate, "code").or_else(|| string_field(candidate, "type"));
            let message = match label {
                Some(label) => format!("OpenAI stream failed ({label}): {detail}"),
                None => format!("OpenAI stream failed: {detail}"),
            };
            return sanitize_provider_message(message);
        }
    }

    // `response.incomplete` reports why it stopped instead of an error object.
    if let Some(reason) = payload
        .get("response")
        .and_then(|value| value.get("incomplete_details"))
        .and_then(|details| string_field(details, "reason"))
    {
        return sanitize_provider_message(format!("OpenAI stream ended early: {reason}"));
    }

    PROVIDER_ERROR_MESSAGE.to_owned()
}

fn provider_transport_message(error: &reqwest::Error) -> String {
    sanitize_provider_message(format!(
        "{PROVIDER_UNAVAILABLE_MESSAGE} ({}): {}",
        reqwest_error_kind(error),
        error_chain(error)
    ))
}

/// reqwest's `Display` omits the underlying cause, which is usually the part
/// that identifies the failure (DNS, TLS, refused connection).
fn error_chain(error: &reqwest::Error) -> String {
    use std::error::Error;

    let mut parts = vec![error.to_string()];
    let mut source = error.source();
    while let Some(current) = source {
        let text = current.to_string();
        if !parts.iter().any(|part| part == &text) {
            parts.push(text);
        }
        source = current.source();
    }

    parts.join(": ")
}

fn sanitize_provider_message(message: String) -> String {
    truncate_message(redact_secrets(message.trim()))
}

/// Provider errors quote the credential they rejected, so strip anything that
/// looks like a key before the text reaches a client or the database.
fn redact_secrets(input: &str) -> String {
    const MARKERS: [&str; 3] = ["sk-", "Bearer ", "bearer "];

    let mut out = String::with_capacity(input.len());
    let mut rest = input;

    loop {
        let found = MARKERS
            .iter()
            .filter_map(|marker| rest.find(marker).map(|index| (index, *marker)))
            .min_by_key(|(index, _)| *index);

        let Some((index, marker)) = found else {
            out.push_str(rest);
            return out;
        };

        out.push_str(&rest[..index]);
        out.push_str(marker);
        out.push_str("***");

        let after = &rest[index + marker.len()..];
        let mut end = after
            .find(|character: char| {
                !(character.is_ascii_alphanumeric()
                    || character == '_'
                    || character == '-'
                    || character == '.')
            })
            .unwrap_or(after.len());
        // Keep sentence punctuation: a key never ends with a dot, but the
        // sentence quoting it usually does.
        while end > 0 && after.as_bytes()[end - 1] == b'.' {
            end -= 1;
        }
        rest = &after[end..];
    }
}

fn truncate_message(mut message: String) -> String {
    const MAX_MESSAGE_BYTES: usize = 500;

    if message.len() > MAX_MESSAGE_BYTES {
        let mut end = MAX_MESSAGE_BYTES;
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message.truncate(end);
        message.push('\u{2026}');
    }

    message
}

#[cfg(test)]
#[path = "../../tests/unit/chat/openai.rs"]
mod tests;

impl ModelProvider for OpenAiClient {
    fn stream(&self, prompt: String) -> mpsc::Receiver<OpenAiEvent> {
        OpenAiClient::stream(self, prompt)
    }
}
