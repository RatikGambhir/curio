//! Ingestion use cases: deduplicate, parse, embed, and persist uploads.
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
    time::Instant,
};

use futures_util::{StreamExt, stream};
use tokio::sync::{Mutex, OwnedMutexGuard};
use tracing::{error, info, warn};

use super::{
    model::{ProcessDocumentsResponse, ProcessedDocument, UploadedDocument},
    persistence::build_document_graph,
};
use crate::domains::documents::{
    formats::{ParsedSourceFile, SourceFile},
    model::{
        Document, DocumentChunk, DocumentError, Embedder, document_id_from_content, sha256_hex,
    },
    store::model::{DocumentGraph, PersistedFileIdentity},
};

const TASK_FAILED_MESSAGE: &str = "The document could not be processed.";
const PARSER_STOPPED_MESSAGE: &str = "The document parser stopped unexpectedly.";
const PREPARE_FAILED_MESSAGE: &str = "The document could not be prepared for storage.";

/// Persistence port for the stored-file aggregate, as ingestion needs it.
pub trait IngestionStore: Clone + Send + Sync + 'static {
    /// The owner's current, non-deleted file whose current version has this content.
    fn find_current_by_hash(
        &self,
        owner_id: &str,
        content_sha256: &str,
    ) -> impl Future<Output = Result<Option<PersistedFileIdentity>, DocumentError>> + Send;

    /// Atomically writes the file, its version bytes, and the version's chunks.
    fn persist(
        &self,
        graph: DocumentGraph,
    ) -> impl Future<Output = Result<PersistedFileIdentity, DocumentError>> + Send;
}

#[derive(Clone)]
pub struct IngestionService<S, E> {
    store: S,
    embedder: E,
    processing_locks: Arc<Mutex<HashMap<String, Weak<Mutex<()>>>>>,
    max_concurrent_documents: usize,
}

impl<S: IngestionStore, E: Embedder> IngestionService<S, E> {
    pub fn new(store: S, embedder: E, max_concurrent_documents: usize) -> Self {
        Self {
            store,
            embedder,
            processing_locks: Arc::new(Mutex::new(HashMap::new())),
            max_concurrent_documents: max_concurrent_documents.max(1),
        }
    }

    /// Processes every upload with bounded concurrency. Per-document failures
    /// are reported in the response rather than failing the whole request.
    pub async fn process(
        &self,
        owner_id: &str,
        files: Vec<UploadedDocument>,
    ) -> ProcessDocumentsResponse {
        let owner_id = owner_id.to_owned();
        let mut indexed_documents = stream::iter(files.into_iter().enumerate())
            .map(|(index, file)| {
                let service = self.clone();
                let owner_id = owner_id.clone();
                let filename = file.filename.clone();
                async move {
                    // A separate task isolates a panicking parser from the batch.
                    let worker = tokio::spawn(async move {
                        service.process_uploaded_document(&owner_id, file).await
                    });
                    let document = worker.await.unwrap_or_else(|join_error| {
                        error!(
                            panicked = join_error.is_panic(),
                            "document processing task failed"
                        );
                        ProcessedDocument::failed(filename, TASK_FAILED_MESSAGE.to_owned())
                    });
                    (index, document)
                }
            })
            .buffer_unordered(self.max_concurrent_documents)
            .collect::<Vec<_>>()
            .await;
        indexed_documents.sort_unstable_by_key(|(index, _)| *index);

        ProcessDocumentsResponse::from_documents(
            indexed_documents
                .into_iter()
                .map(|(_, document)| document)
                .collect(),
        )
    }

    async fn process_uploaded_document(
        &self,
        owner_id: &str,
        file: UploadedDocument,
    ) -> ProcessedDocument {
        let filename = file.filename.clone();
        let content_hash = uploaded_document_content_hash(&file);
        let document_id = document_id_from_content(owner_id, &content_hash);
        let _processing_guard = self.lock_processing(&document_id).await;

        match self
            .store
            .find_current_by_hash(owner_id, &content_hash)
            .await
        {
            Ok(Some(existing)) => {
                return ProcessedDocument::skipped(filename, document_id, existing.file_id);
            }
            Ok(None) => {}
            Err(error) => return ProcessedDocument::failed(filename, error.public_message()),
        }

        match self.ingest(owner_id, file).await {
            Ok((file_id, chunk_count)) => {
                ProcessedDocument::completed(filename, document_id, file_id, chunk_count)
            }
            Err(message) => ProcessedDocument::failed(filename, message),
        }
    }

    async fn ingest(
        &self,
        owner_id: &str,
        file: UploadedDocument,
    ) -> Result<(String, usize), String> {
        let filename = file.filename.clone();
        let file_size_bytes = file.bytes.len();
        let parser_owner_id = owner_id.to_owned();
        let started_at = Instant::now();
        let parsed =
            tokio::task::spawn_blocking(move || parse_document(file, &parser_owner_id)).await;
        let (document, mut chunks, file_bytes) = match parsed {
            Ok(Ok(parsed)) => parsed,
            Ok(Err(reason)) => {
                warn!(
                    file_size_bytes,
                    elapsed_seconds = started_at.elapsed().as_secs_f64(),
                    "document parsing failed"
                );
                return Err(reason);
            }
            Err(join_error) => {
                error!(
                    file_size_bytes,
                    panicked = join_error.is_panic(),
                    "document parser task failed"
                );
                return Err(PARSER_STOPPED_MESSAGE.to_owned());
            }
        };
        info!(
            file_size_bytes,
            chunk_count = chunks.len(),
            elapsed_seconds = started_at.elapsed().as_secs_f64(),
            "document parsed"
        );

        self.embed_chunks(&filename, &mut chunks).await?;
        let chunk_count = chunks.len();
        let graph = build_document_graph(document, chunks, file_bytes).map_err(|reason| {
            error!(%reason, "document graph failed its persistence invariants");
            PREPARE_FAILED_MESSAGE.to_owned()
        })?;
        let identity = self
            .store
            .persist(graph)
            .await
            .map_err(|error| error.public_message())?;

        Ok((identity.file_id, chunk_count))
    }

    async fn embed_chunks(
        &self,
        filename: &str,
        chunks: &mut [DocumentChunk],
    ) -> Result<(), String> {
        if chunks.is_empty() {
            return Ok(());
        }
        let contents = chunks
            .iter()
            .map(|chunk| chunk.text.clone())
            .collect::<Vec<_>>();
        let embeddings = self
            .embedder
            .embed(contents)
            .await
            .map_err(|error| format!("failed to embed chunks for `{filename}`: {error}"))?;
        if chunks.len() != embeddings.len() {
            error!(
                expected = chunks.len(),
                received = embeddings.len(),
                "embedding provider returned the wrong number of vectors"
            );
            return Err(format!(
                "the embedding provider returned an incomplete result for `{filename}`"
            ));
        }
        for (chunk, embedding) in chunks.iter_mut().zip(embeddings) {
            chunk.embedding = Some(embedding);
        }
        Ok(())
    }

    /// Serializes concurrent ingestion of identical content for one owner.
    async fn lock_processing(&self, document_id: &str) -> OwnedMutexGuard<()> {
        let lock = {
            let mut locks = self.processing_locks.lock().await;
            locks.retain(|_, lock| lock.strong_count() > 0);
            match locks.get(document_id).and_then(Weak::upgrade) {
                Some(lock) => lock,
                None => {
                    let lock = Arc::new(Mutex::new(()));
                    locks.insert(document_id.to_string(), Arc::downgrade(&lock));
                    lock
                }
            }
        };
        lock.lock_owned().await
    }
}

fn uploaded_document_content_hash(file: &UploadedDocument) -> String {
    sha256_hex(&file.bytes)
}

fn parse_document(
    file: UploadedDocument,
    owner_id: &str,
) -> Result<(Document, Vec<DocumentChunk>, Vec<u8>), String> {
    let file_bytes = file.bytes.clone();
    let (document, chunks) =
        match SourceFile::from_bytes(file.filename, file.bytes)?.parse(owner_id)? {
            ParsedSourceFile::Pdf(assembly) => (assembly.document, assembly.chunks),
            ParsedSourceFile::Docx(assembly) => (assembly.document, assembly.chunks),
        };
    Ok((document, chunks, file_bytes))
}

#[cfg(test)]
#[path = "../../../../tests/unit/domains/documents/ingestion/service.rs"]
mod tests;
