mod openai;
pub mod protocol;

use std::convert::Infallible;

use axum::{
    Json,
    extract::State,
    response::{
        IntoResponse, Sse,
        sse::{Event, KeepAlive},
    },
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use self::{
    openai::{OpenAiClient, OpenAiEvent},
    protocol::{ChatStreamEvent, ChatStreamRequest},
};

#[derive(Clone)]
pub struct ChatState {
    openai: OpenAiClient,
}

impl ChatState {
    pub fn new(openai_api_key: String, openai_model: String, openai_base_url: String) -> Self {
        Self {
            openai: OpenAiClient::new(openai_api_key, openai_model, openai_base_url),
        }
    }
}

pub async fn stream_chat(
    State(state): State<ChatState>,
    Json(request): Json<ChatStreamRequest>,
) -> impl IntoResponse {
    let (client_sender, client_receiver) = mpsc::channel::<Result<Event, Infallible>>(32);

    tokio::spawn(async move {
        let mut upstream = state.openai.stream(request.prompt.clone());

        while let Some(event) = upstream.recv().await {
            let event = match event {
                OpenAiEvent::Token(token) => ChatStreamEvent::token(&request, token),
                OpenAiEvent::Done(response_id) => ChatStreamEvent::done(&request, response_id),
                OpenAiEvent::Error { code, message } => {
                    ChatStreamEvent::error(&request, code, message)
                }
            };

            let terminal = matches!(event, ChatStreamEvent::Done(_) | ChatStreamEvent::Error(_));
            let Ok(event) = event.into_axum_event() else {
                break;
            };
            if client_sender.send(Ok(event)).await.is_err() || terminal {
                break;
            }
        }
    });

    Sse::new(ReceiverStream::new(client_receiver)).keep_alive(KeepAlive::default())
}
