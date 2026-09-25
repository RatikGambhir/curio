use super::*;
use std::sync::{Arc, Mutex};

type Audit = Arc<Mutex<Vec<String>>>;
struct Store {
    audit: Audit,
    fail_start: bool,
    fail_finish: bool,
}
impl ChatStore for Store {
    async fn begin_chat(&self, _: &ChatTurn) -> Result<(), ChatStorageError> {
        self.audit.lock().unwrap().push("begin".into());
        if self.fail_start {
            Err(ChatStorageError)
        } else {
            Ok(())
        }
    }
    async fn finish_assistant(
        &self,
        _: &ChatTurn,
        content: &str,
        outcome: AssistantOutcome<'_>,
    ) -> Result<(), ChatStorageError> {
        self.audit
            .lock()
            .unwrap()
            .push(format!("persist:{outcome:?}:{content}"));
        if self.fail_finish {
            Err(ChatStorageError)
        } else {
            Ok(())
        }
    }
    async fn list_conversations(&self) -> Result<Vec<ConversationRecord>, ChatStorageError> {
        Ok(Vec::new())
    }
    async fn conversation_messages(&self, _: &str) -> Result<Vec<MessageRecord>, ChatStorageError> {
        Ok(Vec::new())
    }
}
struct Provider {
    audit: Audit,
    events: Mutex<Vec<ModelEvent>>,
}
impl ModelProvider for Provider {
    fn stream(&self, _: String) -> mpsc::Receiver<ModelEvent> {
        self.audit.lock().unwrap().push("provider".into());
        let (sender, receiver) = mpsc::channel(10);
        for event in std::mem::take(&mut *self.events.lock().unwrap()) {
            sender.try_send(event).unwrap();
        }
        receiver
    }
}
struct Sink {
    audit: Audit,
    failure: Option<DeliveryError>,
    fail_terminal_only: bool,
}
impl ChatEventSink for Sink {
    async fn send(&self, event: ChatEvent) -> Result<(), DeliveryError> {
        self.audit.lock().unwrap().push(format!("send:{event:?}"));
        match self.failure {
            Some(failure) if !self.fail_terminal_only || event.is_terminal() => Err(failure),
            _ => Ok(()),
        }
    }
}
fn request() -> ChatTurn {
    ChatTurn {
        conversation_id: "conversation".into(),
        user_message_id: "user".into(),
        assistant_message_id: "assistant".into(),
        prompt: "Hello".into(),
    }
}
fn setup(events: Vec<ModelEvent>) -> (ChatService<Store, Provider>, Sink, Audit) {
    let audit = Audit::default();
    (
        ChatService::new(
            Store {
                audit: audit.clone(),
                fail_start: false,
                fail_finish: false,
            },
            Provider {
                audit: audit.clone(),
                events: Mutex::new(events),
            },
        ),
        Sink {
            audit: audit.clone(),
            failure: None,
            fail_terminal_only: false,
        },
        audit,
    )
}

#[tokio::test]
async fn commits_initial_records_before_provider_and_completion_before_delivery() {
    let (service, sink, audit) = setup(vec![
        ModelEvent::Token("hello".into()),
        ModelEvent::Done("response".into()),
        ModelEvent::Token("ignored".into()),
    ]);
    let session = service.start(request()).await.unwrap();
    service.run(session, &sink).await;
    assert_eq!(
        *audit.lock().unwrap(),
        [
            "begin",
            "provider",
            "send:Token(\"hello\")",
            "persist:Completed { response_id: \"response\" }:hello",
            "send:Done(\"response\")"
        ]
    );
}

#[tokio::test]
async fn failed_start_cannot_call_provider() {
    let (mut service, _, audit) = setup(Vec::new());
    service.repository.fail_start = true;
    assert!(matches!(
        service.start(request()).await,
        Err(ChatEvent::Error {
            code: "storage_error",
            ..
        })
    ));
    assert_eq!(*audit.lock().unwrap(), ["begin"]);
}

#[tokio::test]
async fn provider_failure_persists_partial_content_before_error_delivery() {
    let (service, sink, audit) = setup(vec![
        ModelEvent::Token("partial".into()),
        ModelEvent::Error {
            code: "provider_error",
            message: "Unavailable".into(),
        },
    ]);
    service
        .run(service.start(request()).await.unwrap(), &sink)
        .await;
    let audit = audit.lock().unwrap();
    assert_eq!(
        audit[3],
        "persist:Failed { code: \"provider_error\" }:partial"
    );
    assert_eq!(
        audit[4],
        "send:Error { code: \"provider_error\", message: \"Unavailable\" }"
    );
}

#[tokio::test]
async fn failed_terminal_write_does_not_emit_success() {
    let (mut service, sink, audit) = setup(vec![ModelEvent::Done("response".into())]);
    service.repository.fail_finish = true;
    service
        .run(service.start(request()).await.unwrap(), &sink)
        .await;
    let audit = audit.lock().unwrap();
    assert!(
        audit
            .last()
            .unwrap()
            .starts_with("send:Error { code: \"storage_error\"")
    );
    assert!(!audit.iter().any(|entry| entry.starts_with("send:Done")));
}

#[tokio::test]
async fn disconnect_and_serialization_failure_interrupt_with_buffered_content() {
    for (failure, code) in [
        (DeliveryError::Disconnected, "client_disconnected"),
        (DeliveryError::Serialization, "serialization_error"),
    ] {
        let (service, mut sink, audit) = setup(vec![
            ModelEvent::Token("partial".into()),
            ModelEvent::Done("ignored".into()),
        ]);
        sink.failure = Some(failure);
        service
            .run(service.start(request()).await.unwrap(), &sink)
            .await;
        assert_eq!(
            audit.lock().unwrap().last().unwrap(),
            &format!("persist:Interrupted {{ code: {code:?} }}:partial")
        );
    }
}

#[tokio::test]
async fn unexpected_upstream_end_marks_interrupted_before_error_delivery() {
    let (service, sink, audit) = setup(Vec::new());
    service
        .run(service.start(request()).await.unwrap(), &sink)
        .await;
    let audit = audit.lock().unwrap();
    assert_eq!(audit[2], "persist:Interrupted { code: \"stream_ended\" }:");
    assert!(audit[3].starts_with("send:Error { code: \"incomplete_stream\""));
}

#[tokio::test]
async fn disconnect_after_terminal_commit_does_not_overwrite_completion() {
    let (service, mut sink, audit) = setup(vec![ModelEvent::Done("response".into())]);
    sink.failure = Some(DeliveryError::Disconnected);
    sink.fail_terminal_only = true;
    service
        .run(service.start(request()).await.unwrap(), &sink)
        .await;
    assert!(
        !audit
            .lock()
            .unwrap()
            .iter()
            .any(|entry| entry.contains("Interrupted"))
    );
}
