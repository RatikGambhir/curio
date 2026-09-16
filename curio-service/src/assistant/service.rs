use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{Instrument, error, info_span, warn};

use crate::{
    assistant::{
        model::{
            ChatStreamEvent, ChatStreamRequest, ConversationRecord, EncodedChatEvent, MessageRecord,
        },
        provider::{OpenAiClient, OpenAiEvent},
        repository::AssistantRepository,
    },
    diagnostics,
};

const STREAM_BUFFER_SIZE: usize = 32;

#[derive(Clone)]
pub struct AssistantService {
    provider: OpenAiClient,
    repository: AssistantRepository,
}

impl AssistantService {
    pub fn new(
        openai_api_key: String,
        openai_model: String,
        openai_base_url: String,
        repository: AssistantRepository,
    ) -> Self {
        Self {
            provider: OpenAiClient::new(openai_api_key, openai_model, openai_base_url),
            repository,
        }
    }

    pub async fn stream_chat(
        &self,
        request: ChatStreamRequest,
    ) -> ReceiverStream<EncodedChatEvent> {
        let (client_sender, client_receiver) = mpsc::channel(STREAM_BUFFER_SIZE);
        let chat_span = info_span!(
            "chat_stream",
            conversation_id = ?request.conversation_id,
            user_message_id = ?request.user_message_id,
            assistant_message_id = ?request.assistant_message_id,
        );

        let begin_result = self
            .repository
            .begin_chat(&request)
            .instrument(chat_span.clone())
            .await;
        if let Err(storage_error) = begin_result {
            chat_span.in_scope(|| {
                diagnostics::log_database_error("chat_begin", &storage_error);
            });
            if let Ok(event) = ChatStreamEvent::error(
                &request,
                "storage_error",
                "The conversation could not be saved.",
            )
            .into_encoded()
            {
                let _ = client_sender.send(event).await;
            }

            return ReceiverStream::new(client_receiver);
        }

        let provider = self.provider.clone();
        let repository = self.repository.clone();
        tokio::spawn(
            run_provider_stream(provider, repository, request, client_sender).instrument(chat_span),
        );

        ReceiverStream::new(client_receiver)
    }

    pub async fn list_conversations(&self) -> Result<Vec<ConversationRecord>, sqlx::Error> {
        self.repository.list_conversations().await
    }

    pub async fn conversation_messages(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<MessageRecord>, sqlx::Error> {
        self.repository.conversation_messages(conversation_id).await
    }
}

async fn run_provider_stream(
    provider: OpenAiClient,
    repository: AssistantRepository,
    request: ChatStreamRequest,
    client_sender: mpsc::Sender<EncodedChatEvent>,
) {
    let mut upstream = provider.stream(request.prompt.clone());
    let mut assistant_content = String::new();

    while let Some(provider_event) = upstream.recv().await {
        let event = handle_provider_event(
            &repository,
            &request,
            &mut assistant_content,
            provider_event,
        )
        .await;
        let terminal = matches!(event, ChatStreamEvent::Done(_) | ChatStreamEvent::Error(_));
        let event = match event.into_encoded() {
            Ok(event) => event,
            Err(_) => {
                error!("failed to serialize a chat SSE event");
                interrupt_assistant(
                    &repository,
                    &request,
                    &assistant_content,
                    "serialization_error",
                    "chat_interrupt_after_serialization",
                )
                .await;
                return;
            }
        };

        if client_sender.send(event).await.is_err() {
            if !terminal {
                warn!("chat client disconnected before a terminal event");
                interrupt_assistant(
                    &repository,
                    &request,
                    &assistant_content,
                    "client_disconnected",
                    "chat_interrupt_after_disconnect",
                )
                .await;
            }
            return;
        }
        if terminal {
            return;
        }
    }

    handle_incomplete_stream(&repository, &request, &assistant_content, &client_sender).await;
}

async fn handle_provider_event(
    repository: &AssistantRepository,
    request: &ChatStreamRequest,
    assistant_content: &mut String,
    event: OpenAiEvent,
) -> ChatStreamEvent {
    match event {
        OpenAiEvent::Token(token) => {
            assistant_content.push_str(&token);
            ChatStreamEvent::token(request, token)
        }
        OpenAiEvent::Done(response_id) => match repository
            .complete_assistant(request, assistant_content, &response_id)
            .await
        {
            Ok(()) => ChatStreamEvent::done(request, response_id),
            Err(storage_error) => {
                diagnostics::log_database_error("chat_complete_assistant", &storage_error);
                ChatStreamEvent::error(
                    request,
                    "storage_error",
                    "The completed response could not be saved.",
                )
            }
        },
        OpenAiEvent::Error { code, message } => {
            error!(provider_error_code = code, "model provider stream failed");
            match repository
                .fail_assistant(request, assistant_content, code)
                .await
            {
                Ok(()) => ChatStreamEvent::error(request, code, message),
                Err(storage_error) => {
                    diagnostics::log_database_error("chat_fail_assistant", &storage_error);
                    ChatStreamEvent::error(
                        request,
                        "storage_error",
                        "The failed response state could not be saved.",
                    )
                }
            }
        }
    }
}

async fn handle_incomplete_stream(
    repository: &AssistantRepository,
    request: &ChatStreamRequest,
    assistant_content: &str,
    client_sender: &mpsc::Sender<EncodedChatEvent>,
) {
    error!("model provider event stream ended without a terminal event");
    interrupt_assistant(
        repository,
        request,
        assistant_content,
        "stream_ended",
        "chat_interrupt_after_incomplete_stream",
    )
    .await;
    let event = ChatStreamEvent::error(
        request,
        "incomplete_stream",
        "The response stream ended unexpectedly.",
    );
    match event.into_encoded() {
        Ok(event) => {
            let _ = client_sender.send(event).await;
        }
        Err(_) => {
            error!("failed to serialize the incomplete-stream SSE error");
        }
    }
}

async fn interrupt_assistant(
    repository: &AssistantRepository,
    request: &ChatStreamRequest,
    content: &str,
    error_code: &str,
    operation: &'static str,
) {
    if let Err(storage_error) = repository
        .interrupt_assistant(request, content, error_code)
        .await
    {
        diagnostics::log_database_error(operation, &storage_error);
    }
}
