//! Chat run orchestration: persistence ordering, provider streaming and the
//! public event stream, independent of HTTP.
use std::{sync::Arc, time::Instant};

use tokio::sync::mpsc;
use tracing::{Instrument, Span, error, info, warn};

use crate::adapters::openai::client::{ChatStreamError, OpenAiClient};

pub use super::repository::{ConversationRecord, MessageRecord};

use super::{
    model::{AssistantOutcome, ChatStorageError, ChatStreamEvent, ChatStreamRequest},
    repository::ChatRepository,
};

const EVENT_CHANNEL_CAPACITY: usize = 32;

/// Persistence operations the chat lifecycle needs.
pub trait ChatStore: Clone + Send + Sync + 'static {
    fn start_run(
        &self,
        request: &ChatStreamRequest,
    ) -> impl Future<Output = Result<(), ChatStorageError>> + Send;
    fn finish_run(
        &self,
        request: &ChatStreamRequest,
        content: &str,
        outcome: AssistantOutcome<'_>,
    ) -> impl Future<Output = Result<(), ChatStorageError>> + Send;
    fn list_conversations(
        &self,
    ) -> impl Future<Output = Result<Vec<ConversationRecord>, ChatStorageError>> + Send;
    fn conversation_messages(
        &self,
        conversation_id: &str,
    ) -> impl Future<Output = Result<Vec<MessageRecord>, ChatStorageError>> + Send;
}

/// Streaming model provider: sends text deltas and resolves to the provider
/// response ID, or to a sanitized terminal error.
pub trait ChatModelClient: Send + Sync + 'static {
    fn gen_chat_response_streaming(
        &self,
        prompt: &str,
        model: &str,
        delta_sender: mpsc::Sender<String>,
    ) -> impl Future<Output = Result<String, ChatStreamError>> + Send;
}

impl ChatModelClient for OpenAiClient {
    async fn gen_chat_response_streaming(
        &self,
        prompt: &str,
        model: &str,
        delta_sender: mpsc::Sender<String>,
    ) -> Result<String, ChatStreamError> {
        OpenAiClient::gen_chat_response_streaming(self, prompt, model, delta_sender).await
    }
}

pub struct ChatService<R = ChatRepository, P = OpenAiClient> {
    repository: R,
    openai: Arc<P>,
    model: String,
}

impl<R: ChatStore, P: ChatModelClient> ChatService<R, P> {
    pub fn new(repository: R, openai: Arc<P>, model: String) -> Self {
        Self {
            repository,
            openai,
            model,
        }
    }

    /// Commits the conversation, user message and pending assistant row before
    /// the provider is called. A failed commit yields a single `error` event.
    pub async fn ask(&self, request: ChatStreamRequest) -> mpsc::Receiver<ChatStreamEvent> {
        let (event_tx, event_rx) = mpsc::channel(EVENT_CHANNEL_CAPACITY);
        if self.repository.start_run(&request).await.is_err() {
            let _ = event_tx.try_send(ChatStreamEvent::error(
                &request,
                "storage_error",
                "The conversation could not be saved.",
            ));
            return event_rx;
        }
        info!(model = %self.model, "chat run started");
        tokio::spawn(
            run(
                self.repository.clone(),
                self.openai.clone(),
                self.model.clone(),
                request,
                event_tx,
            )
            .instrument(Span::current()),
        );
        event_rx
    }

    pub async fn list_conversations(&self) -> Result<Vec<ConversationRecord>, ChatStorageError> {
        self.repository.list_conversations().await
    }

    pub async fn conversation_messages(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<MessageRecord>, ChatStorageError> {
        self.repository.conversation_messages(conversation_id).await
    }
}

/// Streams one assistant response. Completed and failed states are committed
/// before their terminal event; a disconnect marks the assistant interrupted.
async fn run<R: ChatStore, P: ChatModelClient>(
    repository: R,
    openai: Arc<P>,
    model: String,
    request: ChatStreamRequest,
    event_tx: mpsc::Sender<ChatStreamEvent>,
) {
    let run_started_at = Instant::now();
    let (delta_tx, mut delta_rx) = mpsc::channel(EVENT_CHANNEL_CAPACITY);
    let response = openai.gen_chat_response_streaming(&request.prompt, &model, delta_tx);
    tokio::pin!(response);
    let mut buffer = String::new();

    loop {
        tokio::select! {
            biased;
            _ = event_tx.closed() => {
                warn!(elapsed_ms = run_started_at.elapsed().as_millis(), "chat client disconnected before a terminal event");
                interrupt(&repository, &request, &buffer, "client_disconnected").await;
                return;
            }
            result = &mut response => {
                while let Ok(delta) = delta_rx.try_recv() {
                    buffer.push_str(&delta);
                    if event_tx.send(ChatStreamEvent::token(&request, delta)).await.is_err() {
                        warn!("chat client disconnected before a terminal event");
                        interrupt(&repository, &request, &buffer, "client_disconnected").await;
                        return;
                    }
                }
                let terminal = match result {
                    Ok(response_id) => {
                        let outcome = AssistantOutcome::Completed { response_id: &response_id };
                        match repository.finish_run(&request, &buffer, outcome).await {
                            Ok(()) => {
                                info!(response_chars = buffer.chars().count(), elapsed_ms = run_started_at.elapsed().as_millis(), "chat run completed");
                                ChatStreamEvent::done(&request, response_id)
                            }
                            Err(ChatStorageError) => ChatStreamEvent::error(
                                &request,
                                "storage_error",
                                "The completed response could not be saved.",
                            ),
                        }
                    }
                    Err(provider_error) => {
                        let code = provider_error.category();
                        error!(provider_error_code = code, elapsed_ms = run_started_at.elapsed().as_millis(), "model provider stream failed");
                        match repository.finish_run(&request, &buffer, AssistantOutcome::Failed { code }).await {
                            Ok(()) => ChatStreamEvent::error(&request, code, provider_error.message()),
                            Err(ChatStorageError) => ChatStreamEvent::error(
                                &request,
                                "storage_error",
                                "The failed response state could not be saved.",
                            ),
                        }
                    }
                };
                // The terminal state is already committed; a late disconnect must not overwrite it.
                let _ = event_tx.send(terminal).await;
                return;
            }
            delta = delta_rx.recv() => {
                let Some(delta) = delta else { continue };
                buffer.push_str(&delta);
                if event_tx.send(ChatStreamEvent::token(&request, delta)).await.is_err() {
                    warn!("chat client disconnected before a terminal event");
                    interrupt(&repository, &request, &buffer, "client_disconnected").await;
                    return;
                }
            }
        }
    }
}

async fn interrupt(
    repository: &impl ChatStore,
    request: &ChatStreamRequest,
    content: &str,
    code: &'static str,
) {
    let _ = repository
        .finish_run(request, content, AssistantOutcome::Interrupted { code })
        .await;
}

#[cfg(test)]
#[path = "../../../tests/unit/domains/chat/service.rs"]
mod tests;
