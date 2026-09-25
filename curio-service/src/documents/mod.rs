//! Documents composition root.
//!
//! Subdomains:
//! - `formats`: PDF/DOCX parsing and token-bounded chunking (plus image,
//!   PowerPoint, and spreadsheet text extractors not yet wired to ingestion);
//! - `ingestion`: uploads, deduplication, embedding, and background jobs;
//! - `store`: the owner-scoped file/version/blob aggregate;
//! - `index`: per-version chunk rows and the chunk read model;
//! - `search`: vector and keyword retrieval;
//! - `viewing`: listing, deletion, chunks, raw text, and PDF previews.
//!
//! Provider (`openai`), converter (`office`), and PostgreSQL adapters meet the
//! use cases only here. Every route requires the bearer middleware, and the
//! authenticated identity is the document owner.
pub mod domain;
pub mod formats;
mod handlers;
mod index;
mod ingestion;
mod office;
mod openai;
mod search;
mod store;
#[cfg(test)]
#[path = "../../tests/unit/documents/repository.rs"]
mod tests;
mod viewing;

use std::time::Duration;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, post},
};

use self::{
    domain::MAX_TOTAL_REQUEST_FILE_BYTES, index::repository::PostgresChunkIndex,
    ingestion::jobs::DocumentJobService, ingestion::service::IngestionService,
    office::OfficeConverter, openai::OpenAiDocumentsClient, search::service::SearchService,
    store::repository::PostgresDocumentStore, viewing::service::StoredDocumentService,
};
use crate::{config::DocumentsConfig, database::Database};

type Ingestion = IngestionService<PostgresDocumentStore, OpenAiDocumentsClient>;
type Jobs = DocumentJobService<PostgresDocumentStore, OpenAiDocumentsClient>;
type Search = SearchService<PostgresChunkIndex, OpenAiDocumentsClient>;
type Viewing = StoredDocumentService<PostgresDocumentStore, PostgresChunkIndex>;

/// Multipart framing headroom on top of the file-byte limit.
const MULTIPART_OVERHEAD_BYTES: usize = 1024 * 1024;
const UPLOAD_BODY_LIMIT: usize = MAX_TOTAL_REQUEST_FILE_BYTES + MULTIPART_OVERHEAD_BYTES;

pub(crate) fn router(
    database: Database,
    openai_api_key: String,
    openai_base_url: String,
    config: &DocumentsConfig,
) -> Router {
    let provider = OpenAiDocumentsClient::new(
        openai_api_key,
        openai_base_url,
        config.embedding_model.clone(),
        config.image_description_model.clone(),
    );
    let files = PostgresDocumentStore::new(database.clone());
    let chunks = PostgresChunkIndex::new(database);

    let ingestion: Ingestion = IngestionService::new(
        files.clone(),
        provider.clone(),
        config.max_concurrent_documents,
    );
    let jobs: Jobs = DocumentJobService::new(
        ingestion.clone(),
        Duration::from_secs(config.completed_job_retention_seconds),
    );
    let search: Search = SearchService::new(chunks.clone(), provider);
    let viewing: Viewing = StoredDocumentService::new(
        files,
        chunks,
        OfficeConverter::new(config.office_executable.clone()),
    );

    let ingestion_routes = Router::new()
        .route(
            "/process",
            post(ingestion::handlers::process_documents)
                .layer(DefaultBodyLimit::max(UPLOAD_BODY_LIMIT)),
        )
        .with_state(ingestion);
    let job_routes = Router::new()
        .route(
            "/jobs",
            post(ingestion::handlers::start_job).layer(DefaultBodyLimit::max(UPLOAD_BODY_LIMIT)),
        )
        .route(
            "/jobs/{job_id}/events",
            get(ingestion::handlers::job_events),
        )
        .with_state(jobs);
    let search_routes = Router::new()
        .route("/search/vector", post(search::handlers::vector_search))
        .route("/search/keyword", post(search::handlers::keyword_search))
        .with_state(search);
    let viewing_routes = Router::new()
        .route("/", get(viewing::handlers::list_documents))
        .route("/{file_id}", delete(viewing::handlers::delete_document))
        .route("/{file_id}/chunks", get(viewing::handlers::document_chunks))
        .route("/{file_id}/text", get(viewing::handlers::document_text))
        .route("/{file_id}/pdf", get(viewing::handlers::document_pdf))
        .with_state(viewing);

    Router::new()
        .nest(
            "/v1/documents",
            ingestion_routes
                .merge(job_routes)
                .merge(search_routes)
                .merge(viewing_routes),
        )
        .route_layer(middleware::from_fn(crate::auth))
}
