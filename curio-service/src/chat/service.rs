//! Chat lifecycle orchestration, independent of HTTP, SQLx and OpenAI.
use tokio::sync::mpsc;
use tracing::{error, warn};

use super::domain::{
    AssistantOutcome, ChatEvent, ChatStorageError, ChatTurn, ConversationRecord, MessageRecord,
    ModelEvent,
};

pub trait ChatStore: Send + Sync {
    fn begin_chat(
        &self,
        request: &ChatTurn,
    ) -> impl Future<Output = Result<(), ChatStorageError>> + Send;
    fn finish_assistant(
        &self,
        request: &ChatTurn,
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

pub trait ModelProvider: Send + Sync {
    fn stream(&self, prompt: String) -> mpsc::Receiver<ModelEvent>;
}

#[derive(Clone, Copy, Debug)]
pub enum DeliveryError {
    Serialization,
    Disconnected,
}

pub trait ChatEventSink: Send + Sync {
    fn send(&self, event: ChatEvent) -> impl Future<Output = Result<(), DeliveryError>> + Send;
}

/// Only `start` can create a session, after committing the initial records.
pub struct ChatSession {
    request: ChatTurn,
}

#[derive(Clone)]
pub struct ChatService<R, P> {
    repository: R,
    provider: P,
}

impl<R: ChatStore, P: ModelProvider> ChatService<R, P> {
    pub fn new(repository: R, provider: P) -> Self {
        Self {
            repository,
            provider,
        }
    }

    pub async fn start(&self, request: ChatTurn) -> Result<ChatSession, ChatEvent> {
        self.repository.begin_chat(&request).await.map_err(|_| {
            ChatEvent::error("storage_error", "The conversation could not be saved.")
        })?;
        Ok(ChatSession { request })
    }

    pub async fn run(&self, session: ChatSession, sink: &impl ChatEventSink) {
        let request = session.request;
        let mut upstream = self.provider.stream(request.prompt.clone());
        let mut content = String::new();
        while let Some(event) = upstream.recv().await {
            let event = match event {
                ModelEvent::Token(token) => {
                    content.push_str(&token);
                    ChatEvent::Token(token)
                }
                ModelEvent::Done(response_id) => {
                    match self
                        .repository
                        .finish_assistant(
                            &request,
                            &content,
                            AssistantOutcome::Completed {
                                response_id: &response_id,
                            },
                        )
                        .await
                    {
                        Ok(()) => ChatEvent::Done(response_id),
                        Err(_) => ChatEvent::error(
                            "storage_error",
                            "The completed response could not be saved.",
                        ),
                    }
                }
                ModelEvent::Error { code, message } => {
                    error!(provider_error_code = code, "model provider stream failed");
                    match self
                        .repository
                        .finish_assistant(&request, &content, AssistantOutcome::Failed { code })
                        .await
                    {
                        Ok(()) => ChatEvent::error(code, message),
                        Err(_) => ChatEvent::error(
                            "storage_error",
                            "The failed response state could not be saved.",
                        ),
                    }
                }
            };
            let terminal = event.is_terminal();
            match sink.send(event).await {
                Ok(()) => {}
                Err(DeliveryError::Serialization) => {
                    error!("failed to serialize a chat SSE event");
                    self.interrupt(&request, &content, "serialization_error")
                        .await;
                    return;
                }
                Err(DeliveryError::Disconnected) => {
                    if !terminal {
                        warn!("chat client disconnected before a terminal event");
                        self.interrupt(&request, &content, "client_disconnected")
                            .await;
                    }
                    return;
                }
            }
            if terminal {
                return;
            }
        }
        error!("model provider event stream ended without a terminal event");
        self.interrupt(&request, &content, "stream_ended").await;
        let _ = sink
            .send(ChatEvent::error(
                "incomplete_stream",
                "The response stream ended unexpectedly.",
            ))
            .await;
    }

    async fn interrupt(&self, request: &ChatTurn, content: &str, code: &'static str) {
        let _ = self
            .repository
            .finish_assistant(request, content, AssistantOutcome::Interrupted { code })
            .await;
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

#[cfg(test)]
#[path = "../../tests/unit/chat/service.rs"]
mod tests;
