mod openai;
pub mod protocol;

use std::convert::Infallible;

use axum::{
    Json,
    extract::{Path, State},
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
use crate::database::{ConversationRecord, Database, MessageRecord};

#[derive(Clone)]
pub struct ChatState {
    openai: OpenAiClient,
    database: Database,
}

impl ChatState {
    pub fn new(
        openai_api_key: String,
        openai_model: String,
        openai_base_url: String,
        database: Database,
    ) -> Self {
        Self {
            openai: OpenAiClient::new(openai_api_key, openai_model, openai_base_url),
            database,
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationsResponse {
    conversations: Vec<ConversationRecord>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessagesResponse {
    conversation_id: String,
    messages: Vec<MessageRecord>,
}

pub async fn stream_chat(
    State(state): State<ChatState>,
    Json(request): Json<ChatStreamRequest>,
) -> impl IntoResponse {
    let (client_sender, client_receiver) = mpsc::channel::<Result<Event, Infallible>>(32);

    if state.database.begin_chat(&request).await.is_err() {
        let event = ChatStreamEvent::error(
            &request,
            "storage_error",
            "The conversation could not be saved.",
        );
        if let Ok(event) = event.into_axum_event() {
            let _ = client_sender.send(Ok(event)).await;
        }

        return Sse::new(ReceiverStream::new(client_receiver)).keep_alive(KeepAlive::default());
    }

    tokio::spawn(async move {
        let mut upstream = state.openai.stream(request.prompt.clone());
        let mut assistant_content = String::new();

        while let Some(event) = upstream.recv().await {
            let event = match event {
                OpenAiEvent::Token(token) => {
                    assistant_content.push_str(&token);
                    ChatStreamEvent::token(&request, token)
                }
                OpenAiEvent::Done(response_id) => {
                    if state
                        .database
                        .complete_assistant(&request, &assistant_content, &response_id)
                        .await
                        .is_err()
                    {
                        ChatStreamEvent::error(
                            &request,
                            "storage_error",
                            "The completed response could not be saved.",
                        )
                    } else {
                        ChatStreamEvent::done(&request, response_id)
                    }
                }
                OpenAiEvent::Error { code, message } => {
                    if state
                        .database
                        .fail_assistant(&request, &assistant_content, code)
                        .await
                        .is_err()
                    {
                        ChatStreamEvent::error(
                            &request,
                            "storage_error",
                            "The failed response state could not be saved.",
                        )
                    } else {
                        ChatStreamEvent::error(&request, code, message)
                    }
                }
            };

            let terminal = matches!(event, ChatStreamEvent::Done(_) | ChatStreamEvent::Error(_));
            let Ok(event) = event.into_axum_event() else {
                let _ = state
                    .database
                    .interrupt_assistant(&request, &assistant_content, "serialization_error")
                    .await;
                return;
            };
            if client_sender.send(Ok(event)).await.is_err() {
                if !terminal {
                    let _ = state
                        .database
                        .interrupt_assistant(&request, &assistant_content, "client_disconnected")
                        .await;
                }
                return;
            }
            if terminal {
                return;
            }
        }

        let _ = state
            .database
            .interrupt_assistant(&request, &assistant_content, "stream_ended")
            .await;
        let event = ChatStreamEvent::error(
            &request,
            "incomplete_stream",
            "The response stream ended unexpectedly.",
        );
        if let Ok(event) = event.into_axum_event() {
            let _ = client_sender.send(Ok(event)).await;
        }
    });

    Sse::new(ReceiverStream::new(client_receiver)).keep_alive(KeepAlive::default())
}

pub async fn list_conversations(
    State(state): State<ChatState>,
) -> Result<Json<ConversationsResponse>, axum::http::StatusCode> {
    state
        .database
        .list_conversations()
        .await
        .map(|conversations| Json(ConversationsResponse { conversations }))
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn conversation_messages(
    State(state): State<ChatState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<ConversationMessagesResponse>, axum::http::StatusCode> {
    state
        .database
        .conversation_messages(&conversation_id)
        .await
        .map(|messages| {
            Json(ConversationMessagesResponse {
                conversation_id,
                messages,
            })
        })
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}
