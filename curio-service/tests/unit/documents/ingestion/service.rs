use super::*;
use crate::documents::ingestion::test_support::{FixedEmbedder, MemoryStore, docx_bytes};

fn upload(filename: &str, bytes: Vec<u8>) -> UploadedDocument {
    UploadedDocument {
        filename: filename.to_string(),
        bytes,
    }
}

fn service(
    store: MemoryStore,
    embedder: FixedEmbedder,
) -> IngestionService<MemoryStore, FixedEmbedder> {
    IngestionService::new(store, embedder, 2)
}

#[test]
fn upload_content_hash_is_stable_until_document_bytes_change() {
    let original = upload("memo.pdf", b"same bytes".to_vec());
    let renamed = upload("renamed.pdf", b"same bytes".to_vec());
    let changed = upload("memo.pdf", b"changed bytes".to_vec());

    let original_hash = uploaded_document_content_hash(&original);
    assert_eq!(original_hash, uploaded_document_content_hash(&renamed));
    assert_ne!(original_hash, uploaded_document_content_hash(&changed));
}

#[tokio::test]
async fn ingests_a_docx_with_embedded_chunks_scoped_to_the_owner() {
    let store = MemoryStore::default();
    let embedder = FixedEmbedder::default();
    let response = service(store.clone(), embedder.clone())
        .process(
            "user-a",
            vec![upload("memo.docx", docx_bytes("Quarterly memo body."))],
        )
        .await;

    assert_eq!(
        (response.total, response.succeeded, response.failed),
        (1, 1, 0)
    );
    let document = &response.documents[0];
    assert!(document.success && !document.skipped);
    assert_eq!(document.chunk_count, 1);

    let graphs = store.graphs.lock().unwrap();
    assert_eq!(graphs.len(), 1);
    assert_eq!(graphs[0].file.owner_id, "user-a");
    assert_eq!(graphs[0].file.display_name, "memo.docx");
    assert_eq!(
        document.file_id.as_deref(),
        Some(graphs[0].file.file_id.as_str())
    );
    assert_eq!(graphs[0].chunks[0].text, "Quarterly memo body.");
    assert_eq!(graphs[0].chunks[0].embedding, vec![20.0, 1.0]);
    assert_eq!(
        embedder.calls.lock().unwrap().as_slice(),
        [vec!["Quarterly memo body.".to_string()]]
    );
}

#[tokio::test]
async fn identical_content_is_skipped_for_the_same_owner_only() {
    let store = MemoryStore::default();
    let ingestion = service(store.clone(), FixedEmbedder::default());
    let bytes = docx_bytes("Repeated body.");

    ingestion
        .process("user-a", vec![upload("first.docx", bytes.clone())])
        .await;
    let repeated = ingestion
        .process("user-a", vec![upload("renamed.docx", bytes.clone())])
        .await;
    let other_owner = ingestion
        .process("user-b", vec![upload("first.docx", bytes)])
        .await;

    assert!(repeated.documents[0].skipped);
    assert_eq!(repeated.skipped, 1);
    assert_eq!(repeated.failed, 0);
    assert!(!other_owner.documents[0].skipped);
    assert_eq!(store.graphs.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn reports_per_document_failures_in_upload_order() {
    let ingestion = service(MemoryStore::default(), FixedEmbedder::default());

    let response = ingestion
        .process(
            "user-a",
            vec![
                upload("broken.pdf", b"not a pdf".to_vec()),
                upload("good.docx", docx_bytes("Readable body.")),
            ],
        )
        .await;

    assert_eq!(
        (response.total, response.succeeded, response.failed),
        (2, 1, 1)
    );
    assert_eq!(response.documents[0].filename, "broken.pdf");
    assert!(!response.documents[0].success);
    assert!(
        response.documents[0]
            .error
            .as_deref()
            .is_some_and(|error| error.contains("PDF"))
    );
    assert_eq!(response.documents[1].filename, "good.docx");
    assert!(response.documents[1].success);
}

#[tokio::test]
async fn embedding_failures_are_reported_without_persisting() {
    let store = MemoryStore::default();
    let embedder = FixedEmbedder {
        error: Some("the provider is temporarily unavailable"),
        ..FixedEmbedder::default()
    };

    let response = service(store.clone(), embedder)
        .process("user-a", vec![upload("memo.docx", docx_bytes("Body."))])
        .await;

    assert_eq!(
        response.documents[0].error.as_deref(),
        Some("failed to embed chunks for `memo.docx`: the provider is temporarily unavailable")
    );
    assert!(store.graphs.lock().unwrap().is_empty());
}

#[tokio::test]
async fn storage_failures_surface_only_their_public_message() {
    let store = MemoryStore {
        fail_persist: Some(DocumentError::UnknownOwner),
        ..MemoryStore::default()
    };

    let response = service(store, FixedEmbedder::default())
        .process("user-a", vec![upload("memo.docx", docx_bytes("Body."))])
        .await;

    assert_eq!(
        response.documents[0].error.as_deref(),
        Some("No profile exists for that user yet.")
    );
}
