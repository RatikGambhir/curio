//! Background single-file jobs and their SSE status streams.
use std::time::Duration;

use axum::http::{StatusCode, header};
use curio_service::app::config::DocumentsConfig;
use serde_json::Value;

use crate::support::{OWNER_A, OWNER_B, TestDocuments, body_bytes, docx_bytes, file, pdf_bytes};

const JOBS: &str = "/v1/documents/jobs";

async fn start_job(t: &TestDocuments, filename: &str, bytes: &[u8]) -> String {
    let (status, body) = t.upload(JOBS, OWNER_A, &[file(filename, bytes)]).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    assert_eq!(body["filename"], filename);
    body["jobId"].as_str().unwrap().to_owned()
}

/// Reads a job stream to its end and returns `(event name, data)` pairs.
async fn read_events(t: &TestDocuments, job_id: &str) -> Vec<(String, Value)> {
    let response = t.get(&format!("{JOBS}/{job_id}/events"), OWNER_A).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/event-stream"
    );
    let body = tokio::time::timeout(Duration::from_secs(30), body_bytes(response))
        .await
        .expect("a job stream ends after its terminal event");
    parse_events(&String::from_utf8(body).unwrap())
}

fn parse_events(stream: &str) -> Vec<(String, Value)> {
    stream
        .split("\n\n")
        .filter_map(|block| {
            let mut name = None;
            let mut data = None;
            for line in block.lines() {
                if let Some(value) = line.strip_prefix("event: ") {
                    name = Some(value.to_owned());
                } else if let Some(value) = line.strip_prefix("data: ") {
                    data = Some(serde_json::from_str(value).unwrap());
                }
            }
            Some((name?, data?))
        })
        .collect()
}

fn terminal(events: &[(String, Value)]) -> &(String, Value) {
    let last = events.last().expect("at least one event");
    assert!(
        events[..events.len() - 1]
            .iter()
            .all(|(name, _)| name == "processing"),
        "only the final event is terminal: {events:?}"
    );
    last
}

#[tokio::test]
async fn a_job_streams_to_completion_and_stores_the_document() {
    let Some(t) = TestDocuments::start("a_job_streams_to_completion_and_stores_the_document").await
    else {
        return;
    };
    let job_id = start_job(&t, "report.docx", &docx_bytes(&["Background report."])).await;

    let events = read_events(&t, &job_id).await;
    let (name, data) = terminal(&events);
    assert_eq!(name, "completed", "{events:?}");
    assert_eq!(data["status"], "completed");
    assert_eq!(data["jobId"], job_id);
    assert_eq!(data["filename"], "report.docx");
    assert_eq!(data["chunkCount"], 1);
    assert!(data.get("error").is_none(), "absent fields are omitted");
    let file_id = data["fileId"].as_str().unwrap();

    let (status, text) = t
        .get_json(&format!("/v1/documents/{file_id}/text"), OWNER_A)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(text["text"], "Background report.");

    // Subscribing again after completion replays the terminal event and ends.
    let replay = read_events(&t, &job_id).await;
    assert_eq!(replay.len(), 1);
    assert_eq!(replay[0].1, *data);

    t.finish().await;
}

#[tokio::test]
async fn a_job_for_an_unreadable_file_fails_without_storing_it() {
    let Some(t) =
        TestDocuments::start("a_job_for_an_unreadable_file_fails_without_storing_it").await
    else {
        return;
    };
    let job_id = start_job(&t, "broken.pdf", b"%PDF-1.5 not really a pdf").await;

    let events = read_events(&t, &job_id).await;
    let (name, data) = terminal(&events);
    assert_eq!(name, "failed", "{events:?}");
    assert!(!data["error"].as_str().unwrap().is_empty());
    assert!(data.get("fileId").is_none());
    assert!(data.get("chunkCount").is_none());
    assert!(t.listed_names(OWNER_A).await.is_empty());

    t.finish().await;
}

#[tokio::test]
async fn a_job_for_existing_content_is_skipped() {
    let Some(t) = TestDocuments::start("a_job_for_existing_content_is_skipped").await else {
        return;
    };
    let bytes = pdf_bytes(&["Already stored."]);
    let file_id = t.ingest(OWNER_A, "stored.pdf", &bytes).await;

    let job_id = start_job(&t, "again.pdf", &bytes).await;
    let events = read_events(&t, &job_id).await;
    let (name, data) = terminal(&events);

    assert_eq!(name, "skipped", "{events:?}");
    assert_eq!(data["fileId"], file_id);
    assert!(data.get("chunkCount").is_none());
    assert_eq!(t.listed_names(OWNER_A).await, ["stored.pdf"]);

    t.finish().await;
}

#[tokio::test]
async fn job_streams_are_private_to_their_owner() {
    let Some(t) = TestDocuments::start("job_streams_are_private_to_their_owner").await else {
        return;
    };
    let job_id = start_job(&t, "private.docx", &docx_bytes(&["Private job."])).await;

    let (foreign, foreign_body) = t
        .get_json(&format!("{JOBS}/{job_id}/events"), OWNER_B)
        .await;
    let (missing, missing_body) = t
        .get_json(
            &format!("{JOBS}/00000000-0000-0000-0000-000000000000/events"),
            OWNER_A,
        )
        .await;

    assert_eq!(foreign, StatusCode::NOT_FOUND);
    assert_eq!(missing, StatusCode::NOT_FOUND);
    assert_eq!(
        foreign_body, missing_body,
        "foreign jobs look exactly like missing ones"
    );

    // The owner still sees the job finish.
    let events = read_events(&t, &job_id).await;
    assert_eq!(terminal(&events).0, "completed");

    t.finish().await;
}

#[tokio::test]
async fn a_job_needs_exactly_one_valid_file() {
    let Some(t) = TestDocuments::start("a_job_needs_exactly_one_valid_file").await else {
        return;
    };
    let docx = docx_bytes(&["Body."]);

    let (two, _) = t
        .upload(
            JOBS,
            OWNER_A,
            &[file("a.docx", &docx), file("b.docx", &docx)],
        )
        .await;
    let (none, _) = t.upload(JOBS, OWNER_A, &[]).await;
    let (unsupported, _) = t.upload(JOBS, OWNER_A, &[file("notes.txt", b"x")]).await;

    assert_eq!(two, StatusCode::BAD_REQUEST);
    assert_eq!(none, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(unsupported, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(t.listed_names(OWNER_A).await.is_empty());

    t.finish().await;
}

#[tokio::test]
async fn finished_jobs_are_forgotten_after_their_retention() {
    let Some(t) = TestDocuments::start_with(
        "finished_jobs_are_forgotten_after_their_retention",
        DocumentsConfig {
            completed_job_retention_seconds: 0,
            ..DocumentsConfig::default()
        },
    )
    .await
    else {
        return;
    };
    let job_id = start_job(&t, "brief.docx", &docx_bytes(&["Short-lived job."])).await;

    // The document is stored even though nobody watched the job.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while t.listed_names(OWNER_A).await.is_empty() {
        assert!(
            tokio::time::Instant::now() < deadline,
            "the job never finished"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    // With zero retention the registry entry disappears right after completion.
    let mut status = StatusCode::OK;
    while tokio::time::Instant::now() < deadline {
        (status, _) = t
            .get_json(&format!("{JOBS}/{job_id}/events"), OWNER_A)
            .await;
        if status == StatusCode::NOT_FOUND {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(status, StatusCode::NOT_FOUND);

    t.finish().await;
}
