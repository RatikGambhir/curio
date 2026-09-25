use super::*;
use crate::documents::ingestion::{
    domain::DocumentJobStatus,
    test_support::{FixedEmbedder, MemoryStore, docx_bytes},
};

fn jobs() -> DocumentJobService<MemoryStore, FixedEmbedder> {
    DocumentJobService::new(
        IngestionService::new(MemoryStore::default(), FixedEmbedder::default(), 1),
        Duration::from_secs(60),
    )
}

async fn terminal_event(mut receiver: watch::Receiver<DocumentJobEvent>) -> DocumentJobEvent {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let event = receiver.borrow_and_update().clone();
            if event.is_terminal() {
                return event;
            }
            receiver.changed().await.unwrap();
        }
    })
    .await
    .expect("the job should finish")
}

#[test]
fn skipped_jobs_are_terminal_skipped_events() {
    let event = DocumentJobEvent::finished(
        "job-1".to_string(),
        ProcessedDocument::skipped(
            "memo.pdf".to_string(),
            "document-1".to_string(),
            "file-1".to_string(),
        ),
    );

    assert_eq!(event.event_name(), "skipped");
    assert!(event.is_terminal());
    assert_eq!(event.document_id.as_deref(), Some("document-1"));
    assert!(event.chunk_count.is_none());
}

#[test]
fn processing_events_are_not_terminal_and_serialize_without_empty_fields() {
    let event = DocumentJobEvent::processing("job-1".to_string(), "memo.pdf".to_string());

    assert!(!event.is_terminal());
    assert_eq!(
        serde_json::to_value(&event).unwrap(),
        serde_json::json!({ "jobId": "job-1", "filename": "memo.pdf", "status": "processing" })
    );
}

#[tokio::test]
async fn a_started_job_reaches_a_completed_event_for_its_owner() {
    let jobs = jobs();
    let started = jobs
        .start(
            "user-a",
            UploadedDocument {
                filename: "memo.docx".to_string(),
                bytes: docx_bytes("Job body."),
            },
        )
        .await;

    let event = terminal_event(jobs.subscribe("user-a", &started.job_id).await.unwrap()).await;

    assert_eq!(event.status, DocumentJobStatus::Completed);
    assert_eq!(event.job_id, started.job_id);
    assert_eq!(event.filename, "memo.docx");
    assert_eq!(event.chunk_count, Some(1));
    assert!(event.file_id.is_some());
}

#[tokio::test]
async fn jobs_are_invisible_to_other_owners() {
    let jobs = jobs();
    let started = jobs
        .start(
            "user-a",
            UploadedDocument {
                filename: "memo.docx".to_string(),
                bytes: docx_bytes("Private job body."),
            },
        )
        .await;

    assert_eq!(
        jobs.subscribe("user-b", &started.job_id).await.unwrap_err(),
        DocumentError::not_found("That document job was not found.")
    );
    assert!(jobs.subscribe("user-a", "missing-job").await.is_err());
}
