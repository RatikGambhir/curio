use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

const DEFAULT_CURIO_CHAT_WORKER_URL: &str = "http://127.0.0.1:8787";
const STREAM_TOKEN_EVENT: &str = "chat-stream-token";
const STREAM_ERROR_EVENT: &str = "chat-stream-error";
const STREAM_DONE_EVENT: &str = "chat-stream-done";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChatStreamRequest {
    user_id: String,
    conversation_id: String,
    user_message_id: String,
    assistant_message_id: String,
    prompt: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct TokenPayload {
    conversation_id: String,
    message_id: String,
    token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ErrorPayload {
    conversation_id: String,
    message_id: String,
    code: String,
    message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct DonePayload {
    conversation_id: String,
    message_id: String,
    response_id: String,
}

#[derive(Debug, PartialEq, Eq)]
enum ServiceEvent {
    Token(TokenPayload),
    Error(ErrorPayload),
    Done(DonePayload),
}

enum EventDisposition {
    Continue,
    Done,
    Error(String),
}

fn worker_endpoint() -> String {
    let base_url = std::env::var("CURIO_CHAT_WORKER_URL")
        .unwrap_or_else(|_| DEFAULT_CURIO_CHAT_WORKER_URL.to_owned());
    format!("{}/v1/chat/stream", base_url.trim_end_matches('/'))
}

fn emit_stream_error(app: &AppHandle, request: &ChatStreamRequest, code: &str, message: &str) {
    let _ = app.emit(
        STREAM_ERROR_EVENT,
        ErrorPayload {
            conversation_id: request.conversation_id.clone(),
            message_id: request.assistant_message_id.clone(),
            code: code.to_owned(),
            message: message.to_owned(),
        },
    );
}

fn fail(
    app: &AppHandle,
    request: &ChatStreamRequest,
    code: &str,
    message: &str,
) -> Result<(), String> {
    emit_stream_error(app, request, code, message);
    Err(message.to_owned())
}

fn forward_event(
    app: &AppHandle,
    request: &ChatStreamRequest,
    event: ServiceEvent,
) -> Result<EventDisposition, String> {
    let correlated = match &event {
        ServiceEvent::Token(payload) => (&payload.conversation_id, &payload.message_id),
        ServiceEvent::Error(payload) => (&payload.conversation_id, &payload.message_id),
        ServiceEvent::Done(payload) => (&payload.conversation_id, &payload.message_id),
    };
    if correlated.0 != &request.conversation_id || correlated.1 != &request.assistant_message_id {
        return Err("Chat worker returned mismatched stream identifiers.".to_owned());
    }

    match event {
        ServiceEvent::Token(payload) => {
            app.emit(STREAM_TOKEN_EVENT, payload)
                .map_err(|_| "Desktop could not deliver a chat token.".to_owned())?;
            Ok(EventDisposition::Continue)
        }
        ServiceEvent::Error(payload) => {
            let message = payload.message.clone();
            app.emit(STREAM_ERROR_EVENT, payload)
                .map_err(|_| "Desktop could not deliver a chat error.".to_owned())?;
            Ok(EventDisposition::Error(message))
        }
        ServiceEvent::Done(payload) => {
            app.emit(STREAM_DONE_EVENT, payload)
                .map_err(|_| "Desktop could not deliver chat completion.".to_owned())?;
            Ok(EventDisposition::Done)
        }
    }
}

#[tauri::command]
async fn stream_chat(app: AppHandle, request: ChatStreamRequest) -> Result<(), String> {
    let response = match reqwest::Client::new()
        .post(worker_endpoint())
        .json(&request)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return fail(
                &app,
                &request,
                "service_unavailable",
                "The Curio chat worker is unavailable.",
            );
        }
    };

    if !response.status().is_success() {
        return fail(
            &app,
            &request,
            "service_error",
            &format!("The Curio chat worker returned HTTP {}.", response.status()),
        );
    }

    let mut parser = ServiceSseParser::default();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(_) => {
                return fail(
                    &app,
                    &request,
                    "service_unavailable",
                    "The connection to the Curio chat worker was interrupted.",
                );
            }
        };

        let events = match parser.push(&chunk) {
            Ok(events) => events,
            Err(()) => {
                return fail(
                    &app,
                    &request,
                    "malformed_service_stream",
                    "The Curio chat worker returned an invalid stream.",
                );
            }
        };

        for event in events {
            match forward_event(&app, &request, event) {
                Ok(EventDisposition::Continue) => {}
                Ok(EventDisposition::Done) => return Ok(()),
                Ok(EventDisposition::Error(message)) => return Err(message),
                Err(_) => {
                    return fail(
                        &app,
                        &request,
                        "malformed_service_stream",
                        "The Curio chat worker returned an invalid stream.",
                    );
                }
            }
        }
    }

    let events = match parser.finish() {
        Ok(events) => events,
        Err(()) => {
            return fail(
                &app,
                &request,
                "incomplete_service_stream",
                "The Curio chat worker stream ended unexpectedly.",
            );
        }
    };

    for event in events {
        match forward_event(&app, &request, event) {
            Ok(EventDisposition::Continue) => {}
            Ok(EventDisposition::Done) => return Ok(()),
            Ok(EventDisposition::Error(message)) => return Err(message),
            Err(_) => {
                return fail(
                    &app,
                    &request,
                    "malformed_service_stream",
                    "The Curio chat worker returned an invalid stream.",
                );
            }
        }
    }

    fail(
        &app,
        &request,
        "incomplete_service_stream",
        "The Curio chat worker stream ended unexpectedly.",
    )
}

#[derive(Default)]
struct ServiceSseParser {
    buffer: Vec<u8>,
    terminal_event_received: bool,
}

impl ServiceSseParser {
    fn push(&mut self, chunk: &[u8]) -> Result<Vec<ServiceEvent>, ()> {
        if self.terminal_event_received && chunk.iter().any(|byte| !byte.is_ascii_whitespace()) {
            return Err(());
        }

        self.buffer.extend_from_slice(chunk);
        let mut events = Vec::new();
        while let Some((boundary_index, boundary_length)) = find_event_boundary(&self.buffer) {
            let event_block = self.buffer[..boundary_index].to_vec();
            self.buffer.drain(..boundary_index + boundary_length);
            if let Some(event) = parse_service_event(&event_block)? {
                self.accept(event, &mut events)?;
            }
        }

        Ok(events)
    }

    fn finish(&mut self) -> Result<Vec<ServiceEvent>, ()> {
        let mut events = Vec::new();
        if self.buffer.iter().any(|byte| !byte.is_ascii_whitespace()) {
            let event_block = std::mem::take(&mut self.buffer);
            if let Some(event) = parse_service_event(&event_block)? {
                self.accept(event, &mut events)?;
            }
        }

        if !self.terminal_event_received {
            return Err(());
        }

        Ok(events)
    }

    fn accept(&mut self, event: ServiceEvent, events: &mut Vec<ServiceEvent>) -> Result<(), ()> {
        if self.terminal_event_received {
            return Err(());
        }
        if matches!(event, ServiceEvent::Error(_) | ServiceEvent::Done(_)) {
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

fn parse_service_event(event_block: &[u8]) -> Result<Option<ServiceEvent>, ()> {
    let event_block = std::str::from_utf8(event_block).map_err(|_| ())?;
    let mut event_name = None;
    let mut data_lines = Vec::new();

    for line in event_block.lines() {
        if let Some(event) = line.strip_prefix("event:") {
            event_name = Some(event.trim());
        } else if let Some(data) = line.strip_prefix("data:") {
            data_lines.push(data.strip_prefix(' ').unwrap_or(data));
        }
    }

    let Some(event_name) = event_name else {
        return if data_lines.is_empty() {
            Ok(None)
        } else {
            Err(())
        };
    };
    if data_lines.is_empty() {
        return Err(());
    }

    let data = data_lines.join("\n");
    match event_name {
        "token" => serde_json::from_str(&data)
            .map(ServiceEvent::Token)
            .map(Some)
            .map_err(|_| ()),
        "error" => serde_json::from_str(&data)
            .map(ServiceEvent::Error)
            .map(Some)
            .map_err(|_| ()),
        "done" => serde_json::from_str(&data)
            .map(ServiceEvent::Done)
            .map(Some)
            .map_err(|_| ()),
        _ => Err(()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![stream_chat])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(event: &str, data: &str) -> String {
        format!("event: {event}\ndata: {data}\n\n")
    }

    #[test]
    fn parser_handles_split_chunks_and_multiple_events() {
        let stream = format!(
            "{}{}",
            block(
                "token",
                r#"{"conversationId":"conversation-1","messageId":"assistant-1","token":"Hel"}"#
            ),
            block(
                "token",
                r#"{"conversationId":"conversation-1","messageId":"assistant-1","token":"lo"}"#
            )
        );
        let mut parser = ServiceSseParser::default();

        assert!(parser.push(&stream.as_bytes()[..23]).unwrap().is_empty());
        assert_eq!(
            parser.push(&stream.as_bytes()[23..]).unwrap(),
            vec![
                ServiceEvent::Token(TokenPayload {
                    conversation_id: "conversation-1".to_owned(),
                    message_id: "assistant-1".to_owned(),
                    token: "Hel".to_owned(),
                }),
                ServiceEvent::Token(TokenPayload {
                    conversation_id: "conversation-1".to_owned(),
                    message_id: "assistant-1".to_owned(),
                    token: "lo".to_owned(),
                })
            ]
        );
    }

    #[test]
    fn parser_accepts_done_and_error_as_terminal_events() {
        let done = block(
            "done",
            r#"{"conversationId":"conversation-1","messageId":"assistant-1","responseId":"response-1"}"#,
        );
        let mut done_parser = ServiceSseParser::default();
        assert!(matches!(
            done_parser.push(done.as_bytes()).unwrap().as_slice(),
            [ServiceEvent::Done(_)]
        ));
        assert!(done_parser.finish().unwrap().is_empty());

        let error = block(
            "error",
            r#"{"conversationId":"conversation-1","messageId":"assistant-1","code":"provider_error","message":"Request failed"}"#,
        );
        let mut error_parser = ServiceSseParser::default();
        assert!(matches!(
            error_parser.push(error.as_bytes()).unwrap().as_slice(),
            [ServiceEvent::Error(_)]
        ));
        assert!(error_parser.finish().unwrap().is_empty());
    }

    #[test]
    fn parser_rejects_malformed_and_truncated_streams() {
        let mut malformed = ServiceSseParser::default();
        assert!(malformed.push(b"event: token\ndata: not-json\n\n").is_err());

        let token = block(
            "token",
            r#"{"conversationId":"conversation-1","messageId":"assistant-1","token":"partial"}"#,
        );
        let mut truncated = ServiceSseParser::default();
        assert!(truncated.push(token.as_bytes()).is_ok());
        assert!(truncated.finish().is_err());
    }
}
