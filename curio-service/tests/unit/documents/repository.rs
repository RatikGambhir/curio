//! PostgreSQL-backed tests for the stored-file aggregate and chunk index.
use crate::{
    documents::{
        domain::{DocumentError, document_id_from_content, file_version_id, sha256_hex},
        index::{
            domain::{IndexedChunk, KeywordSearch, VectorSearch},
            repository::PostgresChunkIndex,
        },
        ingestion::service::IngestionStore,
        search::service::ChunkSearch,
        store::{
            domain::{DocumentGraph, FilePersistence},
            repository::PostgresDocumentStore,
        },
        viewing::service::{DocumentChunks, StoredDocuments},
    },
    postgres_test_support::PostgresFixture,
};

async fn fixture(test_name: &str) -> Option<PostgresFixture> {
    let postgres = PostgresFixture::provision(test_name).await?;
    for id in ["user-a", "user-b"] {
        sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $1, $1 || '@example.com')")
            .bind(id)
            .execute(postgres.database().pool())
            .await
            .unwrap();
    }
    Some(postgres)
}

fn graph(
    owner_id: &str,
    file_id: &str,
    name: &str,
    bytes: &[u8],
    chunks: &[(&str, Vec<f32>)],
) -> DocumentGraph {
    let content_sha256 = sha256_hex(bytes);
    let version_id = file_version_id(file_id, &content_sha256);
    DocumentGraph {
        file: FilePersistence {
            owner_id: owner_id.to_owned(),
            file_id: file_id.to_owned(),
            display_name: name.to_owned(),
            source_uri: None,
            metadata_json: serde_json::json!({
                "documentId": document_id_from_content(owner_id, &content_sha256)
            })
            .to_string(),
            version_id: version_id.clone(),
            mime_type: "application/pdf".to_owned(),
            content_sha256,
            byte_size: bytes.len() as i64,
            file_bytes: bytes.to_vec(),
        },
        chunks: chunks
            .iter()
            .enumerate()
            .map(|(index, (text, embedding))| IndexedChunk {
                chunk_id: sha256_hex(format!("{version_id}\0{index}").as_bytes()),
                owner_id: owner_id.to_owned(),
                file_id: file_id.to_owned(),
                version_id: version_id.clone(),
                chunk_index: index as i64 + 1,
                text: (*text).to_owned(),
                embedding: embedding.clone(),
                chunk_sha256: sha256_hex(text.as_bytes()),
                token_count: 3,
                page_start: Some(1),
                page_end: Some(1),
                char_start: 0,
                char_end: text.len() as i64,
                section_path: String::new(),
            })
            .collect(),
    }
}

#[tokio::test]
async fn persisted_documents_are_listed_loaded_and_deduplicated_per_owner() {
    let Some(postgres) =
        fixture("persisted_documents_are_listed_loaded_and_deduplicated_per_owner").await
    else {
        return;
    };
    let store = PostgresDocumentStore::new(postgres.database().clone());
    let memo = graph(
        "user-a",
        "file-memo",
        "memo.pdf",
        b"%PDF-memo",
        &[("revenue grew", vec![1.0, 0.0])],
    );

    let identity = store.persist(memo.clone()).await.unwrap();
    assert_eq!(identity.file_id, "file-memo");
    assert_eq!(identity.version_id, memo.file.version_id);

    let found = store
        .find_current_by_hash("user-a", &memo.file.content_sha256)
        .await
        .unwrap();
    assert_eq!(found, Some(identity.clone()));
    assert_eq!(
        store
            .find_current_by_hash("user-b", &memo.file.content_sha256)
            .await
            .unwrap(),
        None
    );

    // Re-persisting the same content is idempotent and replaces its chunks.
    let retried = store.persist(memo.clone()).await.unwrap();
    assert_eq!(retried, identity);

    let listed = store.list("user-a").await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].display_name, "memo.pdf");
    assert_eq!(listed[0].version_number, 1);
    assert_eq!(listed[0].byte_size, 9);
    assert!(store.list("user-b").await.unwrap().is_empty());

    let blob = store
        .current_blob("user-a", "file-memo")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(blob.file_bytes, b"%PDF-memo");
    assert_eq!(
        store.current_blob("user-b", "file-memo").await.unwrap(),
        None
    );

    let index = PostgresChunkIndex::new(postgres.database().clone());
    let chunks = index
        .current_chunks("user-a", "file-memo")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].text, "revenue grew");
    assert_eq!(
        index.current_chunks("user-b", "file-memo").await.unwrap(),
        None
    );

    postgres.cleanup().await;
}

#[tokio::test]
async fn aggregate_writes_enforce_owner_and_file_rules() {
    let Some(postgres) = fixture("aggregate_writes_enforce_owner_and_file_rules").await else {
        return;
    };
    let store = PostgresDocumentStore::new(postgres.database().clone());

    let unknown_owner = graph("user-missing", "file-x", "x.pdf", b"%PDF-x", &[]);
    assert_eq!(
        store.persist(unknown_owner).await,
        Err(DocumentError::UnknownOwner)
    );

    store
        .persist(graph("user-a", "file-shared", "a.pdf", b"%PDF-a", &[]))
        .await
        .unwrap();
    assert!(matches!(
        store
            .persist(graph("user-b", "file-shared", "b.pdf", b"%PDF-b", &[]))
            .await,
        Err(DocumentError::Conflict(_))
    ));

    assert!(!store.soft_delete("user-b", "file-shared").await.unwrap());
    assert!(store.soft_delete("user-a", "file-shared").await.unwrap());
    assert!(!store.soft_delete("user-a", "file-shared").await.unwrap());
    assert!(store.list("user-a").await.unwrap().is_empty());
    assert!(matches!(
        store
            .persist(graph("user-a", "file-shared", "a.pdf", b"%PDF-a", &[]))
            .await,
        Err(DocumentError::Conflict(_))
    ));

    postgres.cleanup().await;
}

#[tokio::test]
async fn a_new_content_version_becomes_current_for_the_same_file() {
    let Some(postgres) = fixture("a_new_content_version_becomes_current_for_the_same_file").await
    else {
        return;
    };
    let store = PostgresDocumentStore::new(postgres.database().clone());
    let index = PostgresChunkIndex::new(postgres.database().clone());

    store
        .persist(graph(
            "user-a",
            "file-v",
            "v.pdf",
            b"%PDF-one",
            &[("first draft", vec![1.0])],
        ))
        .await
        .unwrap();
    let second = graph(
        "user-a",
        "file-v",
        "v.pdf",
        b"%PDF-two",
        &[("second draft", vec![1.0])],
    );
    store.persist(second.clone()).await.unwrap();

    let listed = store.list("user-a").await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].version_number, 2);
    assert_eq!(listed[0].version_id, second.file.version_id);
    let chunks = index
        .current_chunks("user-a", "file-v")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(chunks[0].text, "second draft");

    postgres.cleanup().await;
}

#[tokio::test]
async fn searches_rank_only_the_owners_current_visible_chunks() {
    let Some(postgres) = fixture("searches_rank_only_the_owners_current_visible_chunks").await
    else {
        return;
    };
    let store = PostgresDocumentStore::new(postgres.database().clone());
    let index = PostgresChunkIndex::new(postgres.database().clone());

    store
        .persist(graph(
            "user-a",
            "file-finance",
            "finance.pdf",
            b"%PDF-finance",
            &[
                ("Quarterly revenue grew strongly", vec![1.0, 0.0]),
                ("Office relocation schedule", vec![0.0, 1.0]),
            ],
        ))
        .await
        .unwrap();
    store
        .persist(graph(
            "user-b",
            "file-other",
            "other.pdf",
            b"%PDF-other",
            &[("Revenue for another owner", vec![1.0, 0.0])],
        ))
        .await
        .unwrap();
    store
        .persist(graph(
            "user-a",
            "file-deleted",
            "deleted.pdf",
            b"%PDF-deleted",
            &[("Deleted revenue notes", vec![1.0, 0.0])],
        ))
        .await
        .unwrap();
    store.soft_delete("user-a", "file-deleted").await.unwrap();

    let vector = index
        .search_vector(&VectorSearch::new("user-a", vec![0.9, 0.1], 10).unwrap())
        .await
        .unwrap();
    assert_eq!(vector.len(), 2);
    assert_eq!(vector[0].chunk.text, "Quarterly revenue grew strongly");
    assert!(vector[0].distance < vector[1].distance);
    assert_eq!(vector[0].chunk.display_name, "finance.pdf");

    let mismatched_dimension = index
        .search_vector(&VectorSearch::new("user-a", vec![1.0, 0.0, 0.0], 10).unwrap())
        .await
        .unwrap();
    assert!(mismatched_dimension.is_empty());

    let keyword = index
        .search_keyword(&KeywordSearch::new("user-a", "revenue", 10).unwrap())
        .await
        .unwrap();
    assert_eq!(keyword.len(), 1);
    assert_eq!(keyword[0].chunk.file_id, "file-finance");
    assert!(keyword[0].score > 0.0);

    postgres.cleanup().await;
}
