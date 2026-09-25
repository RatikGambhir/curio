use super::*;
use std::sync::Mutex;

type Audit = Arc<Mutex<Vec<String>>>;

#[derive(Clone)]
struct Store {
    audit: Audit,
    fail_start: bool,
    fail_finish: bool,
}
impl ChatStore for Store {
    async fn start_run(&self, _: &ChatStreamRequest) -> Result<(), ChatStorageError> {
        self.audit.lock().unwrap().push("begin".into());
        if self.fail_start {
            Err(ChatStorageError)
        } else {
            Ok(())
        }
    }
    async fn finish_run(
        &self,
        _: &ChatStreamRequest,
        content: &str,
        outcome: AssistantOutcome<'_>,
    ) -> Result<(), ChatStorageError> {
        // Yield so the receiver would observe a terminal event sent before commit.
        tokio::task::yield_now().await;
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

/// Sends `deltas`, then resolves to `result`; `None` never resolves, like a
/// provider that is still streaming.
struct Provider {
    audit: Audit,
    deltas: Vec<&'static str>,
    result: Option<Result<String, ChatStreamError>>,
}
impl ChatModelClient for Provider {
    async fn gen_chat_response_streaming(
        &self,
        _: &str,
        _: &str,
        delta_sender: mpsc::Sender<String>,
    ) -> Result<String, ChatStreamError> {
        self.audit.lock().unwrap().push("provider".into());
        for delta in &self.deltas {
            delta_sender.send((*delta).to_owned()).await.unwrap();
        }
        match self.result.clone() {
            Some(result) => result,
            None => std::future::pending().await,
        }
    }
}

fn request() -> ChatStreamRequest {
    ChatStreamRequest {
        conversation_id: "conversation".into(),
        user_message_id: "user".into(),
        assistant_message_id: "assistant".into(),
        prompt: "Hello".into(),
    }
}

fn store(audit: &Audit) -> Store {
    Store {
        audit: audit.clone(),
        fail_start: false,
        fail_finish: false,
    }
}

fn provider(
    audit: &Audit,
    deltas: Vec<&'static str>,
    result: Option<Result<String, ChatStreamError>>,
) -> Arc<Provider> {
    Arc::new(Provider {
        audit: audit.clone(),
        deltas,
        result,
    })
}

/// Records each delivered event in the audit trail until the stream closes.
async fn drain(mut receiver: mpsc::Receiver<ChatStreamEvent>, audit: &Audit) {
    while let Some(event) = receiver.recv().await {
        let entry = match &event {
            ChatStreamEvent::Token(token) => format!("send:token:{}", token.token),
            ChatStreamEvent::Done(done) => format!("send:done:{}", done.response_id),
            ChatStreamEvent::Error(error) => format!("send:error:{}:{}", error.code, error.message),
        };
        audit.lock().unwrap().push(entry);
    }
}

async fn run_to_end(store: Store, provider: Arc<Provider>, audit: &Audit) {
    let (event_tx, event_rx) = mpsc::channel(EVENT_CHANNEL_CAPACITY);
    let events = drain(event_rx, audit);
    let run = run(store, provider, "model".into(), request(), event_tx);
    tokio::join!(run, events);
}

#[tokio::test]
async fn commits_initial_records_before_provider_and_completion_before_delivery() {
    let audit = Audit::default();
    let service = ChatService::new(
        store(&audit),
        provider(&audit, vec!["hel", "lo"], Some(Ok("response".into()))),
        "model".into(),
    );
    drain(service.ask(request()).await, &audit).await;
    assert_eq!(
        *audit.lock().unwrap(),
        [
            "begin",
            "provider",
            "send:token:hel",
            "send:token:lo",
            "persist:Completed { response_id: \"response\" }:hello",
            "send:done:response"
        ]
    );
}

#[tokio::test]
async fn failed_start_cannot_call_provider() {
    let audit = Audit::default();
    let mut failing_store = store(&audit);
    failing_store.fail_start = true;
    let service = ChatService::new(
        failing_store,
        provider(&audit, Vec::new(), Some(Ok("response".into()))),
        "model".into(),
    );
    drain(service.ask(request()).await, &audit).await;
    assert_eq!(
        *audit.lock().unwrap(),
        [
            "begin",
            "send:error:storage_error:The conversation could not be saved."
        ]
    );
}

#[tokio::test]
async fn provider_failure_persists_partial_content_before_error_delivery() {
    let audit = Audit::default();
    let failure = ChatStreamError::new("provider_error", "Unavailable");
    run_to_end(
        store(&audit),
        provider(&audit, vec!["partial"], Some(Err(failure))),
        &audit,
    )
    .await;
    let audit = audit.lock().unwrap();
    assert_eq!(
        audit[audit.len() - 2],
        "persist:Failed { code: \"provider_error\" }:partial"
    );
    assert_eq!(
        audit[audit.len() - 1],
        "send:error:provider_error:Unavailable"
    );
}

#[tokio::test]
async fn failed_terminal_write_does_not_emit_success() {
    let audit = Audit::default();
    let mut failing_store = store(&audit);
    failing_store.fail_finish = true;
    run_to_end(
        failing_store,
        provider(&audit, Vec::new(), Some(Ok("response".into()))),
        &audit,
    )
    .await;
    let audit = audit.lock().unwrap();
    assert!(
        audit
            .last()
            .unwrap()
            .starts_with("send:error:storage_error")
    );
    assert!(!audit.iter().any(|entry| entry.starts_with("send:done")));
}

#[tokio::test]
async fn disconnect_interrupts_with_buffered_content() {
    let audit = Audit::default();
    let (event_tx, mut event_rx) = mpsc::channel(EVENT_CHANNEL_CAPACITY);
    let task = tokio::spawn(run(
        store(&audit),
        provider(&audit, vec!["partial"], None),
        "model".into(),
        request(),
        event_tx,
    ));
    assert!(matches!(
        event_rx.recv().await,
        Some(ChatStreamEvent::Token(_))
    ));
    drop(event_rx);
    task.await.unwrap();
    assert_eq!(
        audit.lock().unwrap().last().unwrap(),
        "persist:Interrupted { code: \"client_disconnected\" }:partial"
    );
}

#[tokio::test]
async fn disconnect_after_terminal_commit_does_not_overwrite_completion() {
    let audit = Audit::default();
    let (event_tx, event_rx) = mpsc::channel(1);
    // Fill the only slot so the terminal send waits, then disconnect.
    let task = tokio::spawn(run(
        store(&audit),
        provider(&audit, vec!["a"], Some(Ok("response".into()))),
        "model".into(),
        request(),
        event_tx,
    ));
    while !audit
        .lock()
        .unwrap()
        .iter()
        .any(|entry| entry.starts_with("persist:"))
    {
        tokio::task::yield_now().await;
    }
    drop(event_rx);
    task.await.unwrap();
    let audit = audit.lock().unwrap();
    assert!(audit.iter().any(|entry| entry.contains("Completed")));
    assert!(!audit.iter().any(|entry| entry.contains("Interrupted")));
}
