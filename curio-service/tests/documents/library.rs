//! Stored-document reads and lifecycle: text, chunks, previews, and delete.
use axum::http::{StatusCode, header};

use crate::support::{
    OWNER_A, TestDocuments, assert_chunks_index_into, body_bytes, docx_bytes, long_text,
};

#[tokio::test]
async fn docx_text_and_chunks_share_one_canonical_text() {
    let Some(t) = TestDocuments::start("docx_text_and_chunks_share_one_canonical_text").await
    else {
        return;
    };
    let body = long_text("Section text with ünïcödé and emoji 🙂 inside. ", 3);
    let file_id = t
        .ingest(
            OWNER_A,
            "notes.docx",
            &docx_bytes(&["Heading", &body, "Footer line."]),
        )
        .await;

    let (status, text) = t
        .get_json(&format!("/v1/documents/{file_id}/text"), OWNER_A)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(text["fileName"], "notes.docx");
    assert_eq!(text["sourceKind"], "docx");
    let text = text["text"].as_str().unwrap();
    assert!(text.starts_with("Heading\n") && text.ends_with("\nFooter line."));

    let (status, chunks) = t
        .get_json(&format!("/v1/documents/{file_id}/chunks"), OWNER_A)
        .await;
    assert_eq!(status, StatusCode::OK);
    let chunks = chunks["chunks"].as_array().unwrap();
    assert!(chunks.len() > 3);
    assert_chunks_index_into(text, chunks);
    for chunk in chunks {
        assert_eq!(chunk["fileId"], file_id);
        assert_eq!(chunk["displayName"], "notes.docx");
        assert!(chunk["pageStart"].is_null() && chunk["pageEnd"].is_null());
        assert_eq!(chunk["chunkSha256"].as_str().unwrap().len(), 64);
        crate::support::assert_wire_timestamp(&chunk["createdAt"]);
    }

    t.finish().await;
}

#[tokio::test]
async fn docx_previews_fall_back_to_a_text_pdf_without_libreoffice() {
    let Some(t) =
        TestDocuments::start("docx_previews_fall_back_to_a_text_pdf_without_libreoffice").await
    else {
        return;
    };
    // The default test config has no LibreOffice executable.
    let file_id = t
        .ingest(
            OWNER_A,
            "preview.docx",
            &docx_bytes(&["Preview body with “smart quotes” — and a dash."]),
        )
        .await;

    let response = t
        .get(&format!("/v1/documents/{file_id}/pdf"), OWNER_A)
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers().clone();
    assert_eq!(headers[header::CONTENT_TYPE], "application/pdf");
    assert_eq!(headers[header::CONTENT_DISPOSITION], "inline");
    assert_eq!(headers[header::CACHE_CONTROL], "private, no-store");
    assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    let pdf = body_bytes(response).await;
    assert!(pdf.starts_with(b"%PDF-"));
    let rendered = pdf_extract::extract_text_from_mem(&pdf).unwrap();
    assert!(
        rendered.contains("Preview body with \"smart quotes\" - and a dash."),
        "{rendered}"
    );

    t.finish().await;
}

#[tokio::test]
async fn deleting_hides_a_document_everywhere_and_is_not_repeatable() {
    let Some(t) =
        TestDocuments::start("deleting_hides_a_document_everywhere_and_is_not_repeatable").await
    else {
        return;
    };
    let file_id = t
        .ingest(
            OWNER_A,
            "gone.docx",
            &docx_bytes(&["Revenue notes to delete."]),
        )
        .await;
    let kept = t
        .ingest(
            OWNER_A,
            "kept.docx",
            &docx_bytes(&["Revenue notes to keep."]),
        )
        .await;

    let (status, body) = t.delete(&format!("/v1/documents/{file_id}"), OWNER_A).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(body.is_null(), "204 has no body");

    assert_eq!(t.listed_names(OWNER_A).await, ["kept.docx"]);
    for suffix in ["text", "chunks", "pdf"] {
        let (status, body) = t
            .get_json(&format!("/v1/documents/{file_id}/{suffix}"), OWNER_A)
            .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{suffix}");
        assert_eq!(body["error"], "That document was not found.");
    }
    for route in ["keyword", "vector"] {
        let (status, hits) = t
            .post_json(
                &format!("/v1/documents/search/{route}"),
                OWNER_A,
                serde_json::json!({ "queryText": "revenue notes" }),
            )
            .await;
        assert_eq!(status, StatusCode::OK);
        let hits = hits.as_array().unwrap();
        assert!(!hits.is_empty(), "{route}");
        assert!(
            hits.iter().all(|hit| hit["fileId"] == kept),
            "{route}: {hits:?}"
        );
    }

    let (again, _) = t.delete(&format!("/v1/documents/{file_id}"), OWNER_A).await;
    assert_eq!(again, StatusCode::NOT_FOUND);

    t.finish().await;
}

#[tokio::test]
async fn unknown_and_blank_file_ids_are_rejected() {
    let Some(t) = TestDocuments::start("unknown_and_blank_file_ids_are_rejected").await else {
        return;
    };

    for path in [
        "/v1/documents/unknown-file/text",
        "/v1/documents/unknown-file/chunks",
        "/v1/documents/unknown-file/pdf",
    ] {
        assert_eq!(
            t.get_json(path, OWNER_A).await.0,
            StatusCode::NOT_FOUND,
            "{path}"
        );
    }
    assert_eq!(
        t.delete("/v1/documents/unknown-file", OWNER_A).await.0,
        StatusCode::NOT_FOUND
    );

    // A percent-encoded blank id is a validation error, not a lookup.
    for path in ["/v1/documents/%20/text", "/v1/documents/%20/chunks"] {
        let (status, body) = t.get_json(path, OWNER_A).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{path}");
        assert_eq!(body["error"], "fileId is required");
    }
    assert_eq!(
        t.delete("/v1/documents/%20", OWNER_A).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );

    // An empty library is an empty list, not an error.
    assert_eq!(
        t.get_json("/v1/documents", OWNER_A).await,
        (StatusCode::OK, serde_json::json!({ "documents": [] }))
    );

    t.finish().await;
}
