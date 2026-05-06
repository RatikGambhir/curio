use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

const DEFAULT_CHAT_WORKER_URL: &str = "https://api.gettingcurio.com/chat";
const STREAM_TOKEN_EVENT: &str = "chat-stream-token";
const STREAM_ERROR_EVENT: &str = "chat-stream-error";
const STREAM_DONE_EVENT: &str = "chat-stream-done";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChatStreamRequest {
    chat_id: String,
    text: String,
    user_id: Option<String>,
    thread_id: Option<String>,
    assistant_message_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkerRequestBody<'a> {
    user_id: &'a str,
    prompt: &'a str,
    attachments: Option<serde_json::Value>,
    thread_id: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct WorkerStreamChunk {
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WorkerStreamError {
    error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChatStreamPayload {
    chat_id: String,
    assistant_message_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChatStreamTokenPayload {
    chat_id: String,
    assistant_message_id: String,
    token: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChatStreamErrorPayload {
    chat_id: String,
    assistant_message_id: String,
    error: String,
}

fn chat_worker_url() -> String {
    std::env::var("CHAT_WORKER_URL").unwrap_or_else(|_| DEFAULT_CHAT_WORKER_URL.to_string())
}

fn find_event_separator(buffer: &str) -> Option<(usize, usize)> {
    if let Some(index) = buffer.find("\n\n") {
        return Some((index, 2));
    }

    buffer.find("\r\n\r\n").map(|index| (index, 4))
}

fn parse_stream_event(event_block: &str) -> Result<Option<String>, String> {
    let normalized = event_block.trim();
    if normalized.is_empty() {
        return Ok(None);
    }

    if let Some(payload) = normalized.strip_prefix("data:") {
        let payload = payload.trim();
        if payload == "[DONE]" {
            return Ok(None);
        }

        let parsed: WorkerStreamChunk =
            serde_json::from_str(payload).map_err(|error| error.to_string())?;
        return Ok(parsed.token);
    }

    if let Some(payload) = normalized.strip_prefix("ERROR:") {
        let parsed: WorkerStreamError =
            serde_json::from_str(payload.trim()).map_err(|error| error.to_string())?;
        return Err(parsed
            .error
            .unwrap_or_else(|| "Unknown stream error".to_string()));
    }

    Ok(None)
}

fn emit_stream_error(app: &AppHandle, request: &ChatStreamRequest, error: String) {
    let _ = app.emit(
        STREAM_ERROR_EVENT,
        ChatStreamErrorPayload {
            chat_id: request.chat_id.clone(),
            assistant_message_id: request.assistant_message_id.clone(),
            error,
        },
    );
}

fn emit_stream_token(app: &AppHandle, request: &ChatStreamRequest, token: String) {
    let _ = app.emit(
        STREAM_TOKEN_EVENT,
        ChatStreamTokenPayload {
            chat_id: request.chat_id.clone(),
            assistant_message_id: request.assistant_message_id.clone(),
            token,
        },
    );
}

fn emit_stream_done(app: &AppHandle, request: &ChatStreamRequest) {
    let _ = app.emit(
        STREAM_DONE_EVENT,
        ChatStreamPayload {
            chat_id: request.chat_id.clone(),
            assistant_message_id: request.assistant_message_id.clone(),
        },
    );
}

#[tauri::command]
async fn stream_chat(app: AppHandle, request: ChatStreamRequest) -> Result<(), String> {
    let user_id = request
        .user_id
        .as_deref()
        .unwrap_or("266ee938-12db-47d1-9ffd-6d53d0b25808");
    let body = WorkerRequestBody {
        user_id,
        prompt: &request.text,
        attachments: None,
        thread_id: request.thread_id.as_deref(),
    };

    let response = reqwest::Client::new()
        .post(chat_worker_url())
        .json(&body)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();
        let error = format!("HTTP {status}: {response_text}");
        emit_stream_error(&app, &request, error.clone());
        return Err(error);
    }

    let mut stream = response.bytes_stream();
    let mut stream_buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = match chunk_result {
            Ok(chunk) => chunk,
            Err(error) => {
                let error = error.to_string();
                emit_stream_error(&app, &request, error.clone());
                return Err(error);
            }
        };

        stream_buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some((separator_index, separator_length)) = find_event_separator(&stream_buffer) {
            let event_block = stream_buffer[..separator_index].to_string();
            stream_buffer.drain(..separator_index + separator_length);

            match parse_stream_event(&event_block) {
                Ok(Some(token)) if !token.is_empty() => emit_stream_token(&app, &request, token),
                Ok(_) => {}
                Err(error) => {
                    emit_stream_error(&app, &request, error.clone());
                    return Err(error);
                }
            }
        }
    }

    if !stream_buffer.trim().is_empty() {
        match parse_stream_event(&stream_buffer) {
            Ok(Some(token)) if !token.is_empty() => emit_stream_token(&app, &request, token),
            Ok(_) => {}
            Err(error) => {
                emit_stream_error(&app, &request, error.clone());
                return Err(error);
            }
        }
    }

    emit_stream_done(&app, &request);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![stream_chat])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
