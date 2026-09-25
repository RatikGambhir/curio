//! Documents domain.
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
//! The OpenAI and Office adapters live under `crate::adapters`; `app::bootstrap`
//! composes them with these use cases. Every route requires the bearer
//! middleware, and the authenticated identity is the document owner.
pub mod formats;
pub(crate) mod handler;
pub(crate) mod index;
pub(crate) mod ingestion;
pub mod model;
pub(crate) mod route;
pub(crate) mod search;
pub(crate) mod store;
#[cfg(test)]
#[path = "../../../tests/unit/domains/documents/repository.rs"]
mod tests;
pub(crate) mod viewing;

use self::{
    index::repository::ChunkIndex, ingestion::jobs::DocumentJobService,
    ingestion::service::IngestionService, search::service::SearchService,
    store::repository::DocumentStore, viewing::service::StoredDocumentService,
};
use crate::adapters::openai::documents::OpenAiDocumentsClient;

pub(crate) type Ingestion = IngestionService<DocumentStore, OpenAiDocumentsClient>;
pub(crate) type Jobs = DocumentJobService<DocumentStore, OpenAiDocumentsClient>;
pub(crate) type Search = SearchService<ChunkIndex, OpenAiDocumentsClient>;
pub(crate) type Viewing = StoredDocumentService<DocumentStore, ChunkIndex>;
