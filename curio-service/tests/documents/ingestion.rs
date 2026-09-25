//! Batch ingestion through `POST /v1/documents/process`.
use axum::http::StatusCode;
use serde_json::Value;

use crate::support::{
    DOCX_MIME, OWNER_A, OWNER_B, OWNER_WITHOUT_PROFILE, TestDocuments, UPSTREAM_SECRET,
    assert_chunks_index_into, body_bytes, docx_bytes, field, file, long_text, named_file,
    pdf_bytes, upload_request,
};

const PROCESS: &str = "/v1/documents/process";

#[tokio::test]
async fn a_pdf_round_trips_bytes_text_pages_and_offsets() {
    let Some(t) = TestDocuments::start("a_pdf_round_trips_bytes_text_pages_and_offsets").await
    else {
        return;
    };
    let bytes = pdf_bytes(&["Revenue summary page.", "", "Appendix page."]);

    let document = t.process_one(OWNER_A, "summary.pdf", &bytes).await;
    assert_eq!(document["success"], true, "{document}");
    assert_eq!(document["chunkCount"], 1);
    assert!(document["documentId"].as_str().unwrap().len() == 64);
    let file_id = document["fileId"].as_str().unwrap();

    let stored = t
        .get(&format!("/v1/documents/{file_id}/pdf"), OWNER_A)
        .await;
    assert_eq!(stored.status(), StatusCode::OK);
    assert_eq!(
        body_bytes(stored).await,
        bytes,
        "stored PDFs are served unchanged"
    );

    let (_, text) = t
        .get_json(&format!("/v1/documents/{file_id}/text"), OWNER_A)
        .await;
    assert_eq!(text["sourceKind"], "pdf");
    assert_eq!(text["text"], "Revenue summary page.\n\nAppendix page.");

    let (_, chunks) = t
        .get_json(&format!("/v1/documents/{file_id}/chunks"), OWNER_A)
        .await;
    let chunks = chunks["chunks"].as_array().unwrap();
    assert_eq!(chunks[0]["pageStart"], 1);
    assert_eq!(chunks[0]["pageEnd"], 3);
    assert_chunks_index_into(text["text"].as_str().unwrap(), chunks);

    t.finish().await;
}

#[tokio::test]
async fn long_documents_are_embedded_in_batches_and_indexed_in_order() {
    let Some(t) =
        TestDocuments::start("long_documents_are_embedded_in_batches_and_indexed_in_order").await
    else {
        return;
    };
    let body = long_text("Revenue grew steadily across every regional office. ", 70);

    let document = t
        .process_one(OWNER_A, "long.docx", &docx_bytes(&[&body]))
        .await;
    let chunk_count = document["chunkCount"].as_u64().unwrap() as usize;
    assert!(
        chunk_count > 64,
        "need more than one embedding batch: {chunk_count}"
    );

    let batches = t.openai.batch_sizes();
    assert_eq!(batches.iter().sum::<usize>(), chunk_count);
    assert!(batches.iter().all(|size| *size <= 64), "{batches:?}");
    assert_eq!(batches[0], 64);
    assert!(
        t.openai
            .models()
            .iter()
            .all(|model| model == "text-embedding-3-small"),
        "the configured embedding model is sent"
    );
    assert_eq!(t.openai.requests_without_bearer(), 0);

    let file_id = document["fileId"].as_str().unwrap();
    let (_, text) = t
        .get_json(&format!("/v1/documents/{file_id}/text"), OWNER_A)
        .await;
    let (_, chunks) = t
        .get_json(&format!("/v1/documents/{file_id}/chunks"), OWNER_A)
        .await;
    let chunks = chunks["chunks"].as_array().unwrap();
    assert_eq!(chunks.len(), chunk_count);
    assert_chunks_index_into(text["text"].as_str().unwrap(), chunks);

    t.finish().await;
}

#[tokio::test]
async fn batches_report_each_file_in_upload_order() {
    let Some(t) = TestDocuments::start("batches_report_each_file_in_upload_order").await else {
        return;
    };
    let good_docx = docx_bytes(&["Readable body."]);
    let blank_docx = docx_bytes(&["   "]);
    let good_pdf = pdf_bytes(&["Readable PDF."]);
    let empty_pdf = pdf_bytes(&[""]);

    let (status, body) = t
        .upload(
            PROCESS,
            OWNER_A,
            &[
                file("corrupt.pdf", b"%PDF-1.5 this is not a real PDF"),
                file("good.docx", &good_docx),
                file("blank.docx", &blank_docx),
                file("good.pdf", &good_pdf),
                file("image-only.pdf", &empty_pdf),
            ],
        )
        .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["total"], 5);
    assert_eq!(body["succeeded"], 3);
    assert_eq!(body["failed"], 2);
    assert_eq!(body["skipped"], 0);
    let documents = body["documents"].as_array().unwrap();
    let names = documents
        .iter()
        .map(|document| document["filename"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "corrupt.pdf",
            "good.docx",
            "blank.docx",
            "good.pdf",
            "image-only.pdf"
        ]
    );
    assert_failed(&documents[0]);
    assert_failed(&documents[2]);
    assert_eq!(documents[2]["error"], "DOCX did not contain readable text");
    // A PDF without extractable text is stored with zero chunks.
    assert_eq!(documents[4]["success"], true);
    assert_eq!(documents[4]["chunkCount"], 0);

    assert_eq!(
        t.listed_names(OWNER_A).await,
        ["good.docx", "good.pdf", "image-only.pdf"]
    );

    t.finish().await;
}

fn assert_failed(document: &Value) {
    assert_eq!(document["success"], false, "{document}");
    assert_eq!(document["skipped"], false, "{document}");
    assert!(document["fileId"].is_null(), "{document}");
    assert!(document["documentId"].is_null(), "{document}");
    assert!(
        !document["error"].as_str().unwrap().is_empty(),
        "{document}"
    );
}

#[tokio::test]
async fn identical_content_is_deduplicated_per_owner_even_within_one_batch() {
    let Some(t) =
        TestDocuments::start("identical_content_is_deduplicated_per_owner_even_within_one_batch")
            .await
    else {
        return;
    };
    let bytes = docx_bytes(&["Duplicated body."]);

    // Both copies run concurrently; the per-content lock lets exactly one win.
    let (_, body) = t
        .upload(
            PROCESS,
            OWNER_A,
            &[file("first.docx", &bytes), file("second.docx", &bytes)],
        )
        .await;
    assert_eq!(
        (body["succeeded"].as_u64(), body["skipped"].as_u64()),
        (Some(2), Some(1))
    );
    let documents = body["documents"].as_array().unwrap();
    assert_eq!(documents[0]["fileId"], documents[1]["fileId"]);
    assert_eq!(documents[0]["documentId"], documents[1]["documentId"]);
    assert_eq!(t.listed_names(OWNER_A).await.len(), 1);

    // A later rename is still the same content and keeps the original name.
    let renamed = t.process_one(OWNER_A, "renamed.docx", &bytes).await;
    assert_eq!(renamed["skipped"], true);
    assert_eq!(renamed["fileId"], documents[0]["fileId"]);
    assert_eq!(renamed["chunkCount"], 0);
    assert_eq!(t.listed_names(OWNER_A).await.len(), 1);

    // Another owner gets an independent copy with a different identity.
    let other = t.process_one(OWNER_B, "first.docx", &bytes).await;
    assert_eq!(other["skipped"], false);
    assert_ne!(other["fileId"], documents[0]["fileId"]);
    assert_ne!(other["documentId"], documents[0]["documentId"]);

    t.finish().await;
}

#[tokio::test]
async fn re_uploading_deleted_content_creates_a_fresh_document() {
    let Some(t) =
        TestDocuments::start("re_uploading_deleted_content_creates_a_fresh_document").await
    else {
        return;
    };
    let bytes = docx_bytes(&["Deleted then restored."]);
    let original = t.ingest(OWNER_A, "memo.docx", &bytes).await;

    let (status, _) = t
        .delete(&format!("/v1/documents/{original}"), OWNER_A)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let restored = t.ingest(OWNER_A, "memo.docx", &bytes).await;

    assert_ne!(restored, original);
    assert_eq!(t.listed_names(OWNER_A).await, ["memo.docx"]);
    let (status, _) = t
        .get_json(&format!("/v1/documents/{original}/text"), OWNER_A)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    t.finish().await;
}

#[tokio::test]
async fn an_owner_without_a_profile_gets_a_per_file_error() {
    let Some(t) = TestDocuments::start("an_owner_without_a_profile_gets_a_per_file_error").await
    else {
        return;
    };

    let document = t
        .process_one(OWNER_WITHOUT_PROFILE, "memo.docx", &docx_bytes(&["Body."]))
        .await;

    assert_failed(&document);
    assert_eq!(document["error"], "No profile exists for that user yet.");
    assert!(t.listed_names(OWNER_WITHOUT_PROFILE).await.is_empty());

    t.finish().await;
}

#[tokio::test]
async fn provider_failures_are_sanitized_and_leave_nothing_behind() {
    let Some(t) =
        TestDocuments::start("provider_failures_are_sanitized_and_leave_nothing_behind").await
    else {
        return;
    };
    let bytes = docx_bytes(&["Provider outage body."]);

    for (status, message) in [
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "the provider is temporarily unavailable",
        ),
        (
            StatusCode::TOO_MANY_REQUESTS,
            "the provider is rate limiting requests",
        ),
        (
            StatusCode::UNAUTHORIZED,
            "the provider rejected the service credentials",
        ),
    ] {
        t.openai.fail_with(Some(status));
        let document = t.process_one(OWNER_A, "memo.docx", &bytes).await;
        assert_failed(&document);
        let error = document["error"].as_str().unwrap();
        assert_eq!(
            error,
            format!("failed to embed chunks for `memo.docx`: {message}")
        );
        assert!(!error.contains(UPSTREAM_SECRET));
    }
    assert!(t.listed_names(OWNER_A).await.is_empty());

    // Recovery: nothing partial was stored, so the retry is a fresh ingest.
    t.openai.fail_with(None);
    t.ingest(OWNER_A, "memo.docx", &bytes).await;
    assert_eq!(t.listed_names(OWNER_A).await, ["memo.docx"]);

    t.finish().await;
}

#[tokio::test]
async fn malformed_uploads_are_rejected_before_processing() {
    let Some(t) = TestDocuments::start("malformed_uploads_are_rejected_before_processing").await
    else {
        return;
    };
    let docx = docx_bytes(&["Body."]);

    let cases: Vec<(&str, Vec<crate::support::Part<'_>>, StatusCode)> = vec![
        (
            "owner in a form field",
            vec![field("userId", b"user-b"), file("memo.docx", &docx)],
            StatusCode::BAD_REQUEST,
        ),
        (
            "owner under another name",
            vec![file("memo.docx", &docx), field("ownerId", b"user-b")],
            StatusCode::BAD_REQUEST,
        ),
        ("no parts", vec![], StatusCode::UNPROCESSABLE_ENTITY),
        (
            "only unrelated fields",
            vec![field("note", b"hello")],
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "file in the wrong field",
            vec![named_file("attachment", "memo.docx", &docx)],
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "missing filename",
            vec![field("files", &docx)],
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "blank filename",
            vec![file("   ", &docx)],
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "padded filename",
            vec![file(" memo.docx", &docx)],
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "unsupported type",
            vec![file("notes.txt", b"plain text")],
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "legacy Word document",
            vec![file("old.doc", b"binary")],
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "empty file",
            vec![file("empty.pdf", b"")],
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "one bad file rejects the whole request",
            vec![file("memo.docx", &docx), file("notes.txt", b"plain")],
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
    ];
    for (case, parts, expected) in cases {
        let (status, body) = t.upload(PROCESS, OWNER_A, &parts).await;
        assert_eq!(status, expected, "{case}: {body}");
        assert!(body["error"].is_string(), "{case}: {body}");
    }
    assert!(
        t.listed_names(OWNER_A).await.is_empty(),
        "nothing was stored"
    );

    // Unrelated fields beside a valid file are ignored.
    let (status, body) = t
        .upload(
            PROCESS,
            OWNER_A,
            &[field("note", b"ignored"), file("memo.docx", &docx)],
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["succeeded"], 1);

    // A request that is not multipart at all is a framework rejection.
    let (status, _) = t
        .post_json(PROCESS, OWNER_A, serde_json::json!({ "files": [] }))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    t.finish().await;
}

#[tokio::test]
async fn upload_size_limits_are_enforced() {
    let Some(t) = TestDocuments::start("upload_size_limits_are_enforced").await else {
        return;
    };
    let fifty_megabytes = 50 * 1024 * 1024;

    // One byte over the per-file limit, but within the body limit.
    let oversized_file = vec![b'x'; fifty_megabytes + 1];
    let (status, body) = t
        .upload(PROCESS, OWNER_A, &[file("big.pdf", &oversized_file)])
        .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{body}");
    assert_eq!(
        body["error"],
        "The upload `big.pdf` exceeds the 50 MB file limit."
    );
    drop(oversized_file);

    // Two files that each fit but together exceed the request limit.
    let half = vec![b'x'; fifty_megabytes / 2 + 1024];
    let (status, _) = t
        .upload(
            PROCESS,
            OWNER_A,
            &[file("a.pdf", &half), file("b.pdf", &half)],
        )
        .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    drop(half);

    // A body beyond the transport limit is cut off while streaming.
    let huge = vec![b'x'; fifty_megabytes + 2 * 1024 * 1024];
    let response = t
        .send(upload_request(
            PROCESS,
            OWNER_A,
            crate::support::multipart(&[file("huge.pdf", &huge)]),
        ))
        .await;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);

    assert!(t.listed_names(OWNER_A).await.is_empty());

    t.finish().await;
}

#[tokio::test]
async fn listed_metadata_describes_the_current_version() {
    let Some(t) = TestDocuments::start("listed_metadata_describes_the_current_version").await
    else {
        return;
    };
    let docx = docx_bytes(&["Metadata body."]);
    let file_id = t.ingest(OWNER_A, "b-memo.docx", &docx).await;
    t.ingest(OWNER_A, "a-summary.pdf", &pdf_bytes(&["Summary."]))
        .await;

    let (status, body) = t.get_json("/v1/documents", OWNER_A).await;
    assert_eq!(status, StatusCode::OK);
    let documents = body["documents"].as_array().unwrap();
    assert_eq!(documents.len(), 2);
    assert_eq!(
        documents[0]["displayName"], "a-summary.pdf",
        "sorted by name"
    );
    assert_eq!(documents[0]["mimeType"], "application/pdf");
    let memo = &documents[1];
    assert_eq!(memo["fileId"], file_id);
    assert_eq!(memo["mimeType"], DOCX_MIME);
    assert_eq!(memo["byteSize"], docx.len());
    assert_eq!(memo["versionNumber"], 1);
    assert_eq!(memo["versionId"].as_str().unwrap().len(), 64);
    crate::support::assert_wire_timestamp(&memo["createdAt"]);
    crate::support::assert_wire_timestamp(&memo["updatedAt"]);

    t.finish().await;
}
