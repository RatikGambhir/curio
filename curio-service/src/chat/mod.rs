mod openai;
pub mod protocol;
pub(crate) mod repository;

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
use tracing::{Instrument, error, info_span, warn};

use crate::diagnostics;

use self::{
    openai::{OpenAiClient, OpenAiEvent},
    protocol::{ChatStreamEvent, ChatStreamRequest},
    repository::{ChatRepository, ConversationRecord, MessageRecord},
};

#[derive(Clone)]
pub struct ChatState {
    openai: OpenAiClient,
    repository: ChatRepository,
}

impl ChatState {
    pub fn new(
        openai_api_key: String,
        openai_model: String,
        openai_base_url: String,
        repository: ChatRepository,
    ) -> Self {
        Self {
            openai: OpenAiClient::new(openai_api_key, openai_model, openai_base_url),
            repository,
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
    let chat_span = info_span!(
        "chat_stream",
        conversation_id = ?request.conversation_id,
        user_message_id = ?request.user_message_id,
        assistant_message_id = ?request.assistant_message_id,
    );

    let begin_result = state
        .repository
        .begin_chat(&request)
        .instrument(chat_span.clone())
        .await;
    if let Err(storage_error) = begin_result {
        chat_span.in_scope(|| {
            diagnostics::log_database_error("chat_begin", &storage_error);
        });
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

    tokio::spawn(
        async move {
            let mut upstream = state.openai.stream(request.prompt.clone());
            let mut assistant_content = String::new();

            while let Some(event) = upstream.recv().await {
                let event = match event {
                    OpenAiEvent::Token(token) => {
                        assistant_content.push_str(&token);
                        ChatStreamEvent::token(&request, token)
                    }
                    OpenAiEvent::Done(response_id) => {
                        match state
                            .repository
                            .complete_assistant(&request, &assistant_content, &response_id)
                            .await
                        {
                            Ok(()) => ChatStreamEvent::done(&request, response_id),
                            Err(storage_error) => {
                                diagnostics::log_database_error(
                                    "chat_complete_assistant",
                                    &storage_error,
                                );
                                ChatStreamEvent::error(
                                    &request,
                                    "storage_error",
                                    "The completed response could not be saved.",
                                )
                            }
                        }
                    }
                    OpenAiEvent::Error { code, message } => {
                        error!(provider_error_code = code, "model provider stream failed");
                        match state
                            .repository
                            .fail_assistant(&request, &assistant_content, code)
                            .await
                        {
                            Ok(()) => ChatStreamEvent::error(&request, code, message),
                            Err(storage_error) => {
                                diagnostics::log_database_error(
                                    "chat_fail_assistant",
                                    &storage_error,
                                );
                                ChatStreamEvent::error(
                                    &request,
                                    "storage_error",
                                    "The failed response state could not be saved.",
                                )
                            }
                        }
                    }
                };

                let terminal =
                    matches!(event, ChatStreamEvent::Done(_) | ChatStreamEvent::Error(_));
                let event = match event.into_axum_event() {
                    Ok(event) => event,
                    Err(_) => {
                        error!("failed to serialize a chat SSE event");
                        if let Err(storage_error) = state
                            .repository
                            .interrupt_assistant(
                                &request,
                                &assistant_content,
                                "serialization_error",
                            )
                            .await
                        {
                            diagnostics::log_database_error(
                                "chat_interrupt_after_serialization",
                                &storage_error,
                            );
                        }
                        return;
                    }
                };
                if client_sender.send(Ok(event)).await.is_err() {
                    if !terminal {
                        warn!("chat client disconnected before a terminal event");
                        if let Err(storage_error) = state
                            .repository
                            .interrupt_assistant(
                                &request,
                                &assistant_content,
                                "client_disconnected",
                            )
                            .await
                        {
                            diagnostics::log_database_error(
                                "chat_interrupt_after_disconnect",
                                &storage_error,
                            );
                        }
                    }
                    return;
                }
                if terminal {
                    return;
                }
            }

            error!("model provider event stream ended without a terminal event");
            if let Err(storage_error) = state
                .repository
                .interrupt_assistant(&request, &assistant_content, "stream_ended")
                .await
            {
                diagnostics::log_database_error(
                    "chat_interrupt_after_incomplete_stream",
                    &storage_error,
                );
            }
            let event = ChatStreamEvent::error(
                &request,
                "incomplete_stream",
                "The response stream ended unexpectedly.",
            );
            match event.into_axum_event() {
                Ok(event) => {
                    let _ = client_sender.send(Ok(event)).await;
                }
                Err(_) => {
                    error!("failed to serialize the incomplete-stream SSE error");
                }
            }
        }
        .instrument(chat_span),
    );

    Sse::new(ReceiverStream::new(client_receiver)).keep_alive(KeepAlive::default())
}

pub async fn list_conversations(
    State(state): State<ChatState>,
) -> Result<Json<ConversationsResponse>, axum::http::StatusCode> {
    match state.repository.list_conversations().await {
        Ok(conversations) => Ok(Json(ConversationsResponse { conversations })),
        Err(storage_error) => {
            diagnostics::log_database_error("chat_list_conversations", &storage_error);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn conversation_messages(
    State(state): State<ChatState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<ConversationMessagesResponse>, axum::http::StatusCode> {
    let history_span = info_span!(
        "conversation_messages",
        conversation_id = ?conversation_id,
    );
    let result = state
        .repository
        .conversation_messages(&conversation_id)
        .instrument(history_span.clone())
        .await;

    match result {
        Ok(messages) => Ok(Json(ConversationMessagesResponse {
            conversation_id,
            messages,
        })),
        Err(storage_error) => {
            history_span.in_scope(|| {
                diagnostics::log_database_error("chat_conversation_messages", &storage_error);
            });
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
