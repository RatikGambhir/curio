use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Mutex};
use tauri::{ipc::Channel, State};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChatStreamRequest {
    user_id: String,
    conversation_id: String,
    user_message_id: String,
    assistant_message_id: String,
    prompt: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum StreamPacket {
    Started { status: u16 },
    Chunk { bytes: Vec<u8> },
    End,
    Error { code: String, message: String },
}

#[derive(Default)]
struct InFlightRequests {
    requests: Mutex<HashMap<String, CancellationToken>>,
}

impl InFlightRequests {
    fn register(&self, request_id: String, token: CancellationToken) -> Result<(), String> {
        let mut requests = self
            .requests
            .lock()
            .map_err(|_| "Desktop chat cancellation state is unavailable.".to_owned())?;
        if requests.contains_key(&request_id) {
            return Err("A chat request with this identifier is already active.".to_owned());
        }
        requests.insert(request_id, token);
        Ok(())
    }

    fn cancel(&self, request_id: &str) -> Result<bool, String> {
        let token = self
            .requests
            .lock()
            .map_err(|_| "Desktop chat cancellation state is unavailable.".to_owned())?
            .remove(request_id);
        if let Some(token) = token {
            token.cancel();
            return Ok(true);
        }
        Ok(false)
    }

    fn remove(&self, request_id: &str) -> Result<(), String> {
        self.requests
            .lock()
            .map_err(|_| "Desktop chat cancellation state is unavailable.".to_owned())?
            .remove(request_id);
        Ok(())
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.requests.lock().expect("in-flight lock").len()
    }
}

fn worker_endpoint(worker_url: &str) -> Result<reqwest::Url, String> {
    let mut url = reqwest::Url::parse(worker_url)
        .map_err(|_| "The Curio chat worker URL is invalid.".to_owned())?;
    let is_local_http =
        url.scheme() == "http" && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"));
    if url.scheme() != "https" && !(cfg!(debug_assertions) && is_local_http) {
        return Err("The Curio chat worker URL must use HTTPS.".to_owned());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("The Curio chat worker URL cannot include credentials.".to_owned());
    }
    url.set_path("/v1/chat/stream");
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

fn send_packet(channel: &Channel<StreamPacket>, packet: StreamPacket) -> Result<(), String> {
    channel
        .send(packet)
        .map_err(|_| "Desktop could not deliver a chat stream packet.".to_owned())
}

fn send_transport_error(
    channel: &Channel<StreamPacket>,
    code: &str,
    message: &str,
) -> Result<(), String> {
    send_packet(
        channel,
        StreamPacket::Error {
            code: code.to_owned(),
            message: message.to_owned(),
        },
    )
}

async fn forward_chat(
    worker_url: &str,
    request: &ChatStreamRequest,
    channel: &Channel<StreamPacket>,
    cancellation: CancellationToken,
) -> Result<(), String> {
    let endpoint = worker_endpoint(worker_url)?;
    let response = tokio::select! {
        _ = cancellation.cancelled() => {
            return send_transport_error(channel, "canceled", "The chat response was canceled.");
        }
        response = reqwest::Client::new().post(endpoint).json(request).send() => {
            match response {
                Ok(response) => response,
                Err(_) => {
                    return send_transport_error(
                        channel,
                        "service_unavailable",
                        "The Curio chat worker is unavailable.",
                    );
                }
            }
        }
    };

    send_packet(
        channel,
        StreamPacket::Started {
            status: response.status().as_u16(),
        },
    )?;

    let mut stream = response.bytes_stream();
    loop {
        let next = tokio::select! {
            _ = cancellation.cancelled() => {
                return send_transport_error(channel, "canceled", "The chat response was canceled.");
            }
            next = stream.next() => next,
        };

        match next {
            Some(Ok(bytes)) => {
                send_packet(
                    channel,
                    StreamPacket::Chunk {
                        bytes: bytes.to_vec(),
                    },
                )?;
            }
            Some(Err(_)) => {
                return send_transport_error(
                    channel,
                    "service_unavailable",
                    "The connection to the Curio chat worker was interrupted.",
                );
            }
            None => return send_packet(channel, StreamPacket::End),
        }
    }
}

#[tauri::command]
async fn stream_chat(
    state: State<'_, InFlightRequests>,
    request_id: String,
    worker_url: String,
    request: ChatStreamRequest,
    on_packet: Channel<StreamPacket>,
) -> Result<(), String> {
    if request_id.trim().is_empty() {
        return Err("A chat request identifier is required.".to_owned());
    }

    let cancellation = CancellationToken::new();
    state.register(request_id.clone(), cancellation.clone())?;
    let result = forward_chat(&worker_url, &request, &on_packet, cancellation).await;
    let cleanup_result = state.remove(&request_id);

    match (result, cleanup_result) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Ok(()), Ok(())) => Ok(()),
    }
}

#[tauri::command]
fn cancel_chat(state: State<'_, InFlightRequests>, request_id: String) -> Result<bool, String> {
    state.cancel(&request_id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(InFlightRequests::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![stream_chat, cancel_chat])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_endpoint_is_scoped_to_chat_and_rejects_unsafe_urls() {
        let endpoint = worker_endpoint("https://worker.example/base?token=secret#fragment")
            .expect("valid worker URL");
        assert_eq!(endpoint.as_str(), "https://worker.example/v1/chat/stream");
        assert!(worker_endpoint("file:///tmp/chat").is_err());
        assert!(worker_endpoint("https://user:secret@worker.example").is_err());
    }

    #[test]
    fn packet_serialization_preserves_status_and_arbitrary_bytes() {
        let started =
            serde_json::to_value(StreamPacket::Started { status: 429 }).expect("serialize status");
        let chunk = serde_json::to_value(StreamPacket::Chunk {
            bytes: vec![0, 159, 146, 150, 255],
        })
        .expect("serialize bytes");

        assert_eq!(
            started,
            serde_json::json!({ "type": "started", "status": 429 })
        );
        assert_eq!(
            chunk,
            serde_json::json!({ "type": "chunk", "bytes": [0, 159, 146, 150, 255] })
        );
    }

    #[test]
    fn cancellation_removes_every_in_flight_request() {
        let state = InFlightRequests::default();
        let first = CancellationToken::new();
        let second = CancellationToken::new();
        state
            .register("first".to_owned(), first.clone())
            .expect("register first");
        state
            .register("second".to_owned(), second.clone())
            .expect("register second");

        assert_eq!(state.len(), 2);
        assert!(state.cancel("first").expect("cancel first"));
        assert!(first.is_cancelled());
        assert_eq!(state.len(), 1);
        state.remove("second").expect("remove second");
        assert_eq!(state.len(), 0);
        assert!(!state.cancel("missing").expect("cancel missing"));
    }
}
