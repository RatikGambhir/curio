//! The stored-file aggregate: a logical file, its content versions, each
//! version's bytes, and the indexed chunks of that version.
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domains::documents::index::model::IndexedChunk;

/// Identity of a committed file version, read back from the transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedFileIdentity {
    pub file_id: String,
    pub owner_id: String,
    pub display_name: String,
    pub version_id: String,
}

/// Fully validated values used to persist one logical file version.
///
/// The repository supplies the transaction-derived version number; every other
/// value is represented here before the transaction begins.
#[derive(Debug, Clone)]
pub struct FilePersistence {
    pub(crate) owner_id: String,
    pub(crate) file_id: String,
    pub(crate) display_name: String,
    pub(crate) source_uri: Option<String>,
    pub(crate) metadata_json: String,
    pub(crate) version_id: String,
    pub(crate) mime_type: String,
    pub(crate) content_sha256: String,
    pub(crate) byte_size: i64,
    pub(crate) file_bytes: Vec<u8>,
}

/// One atomic aggregate write: the file version, its bytes, and its chunks.
#[derive(Debug, Clone)]
pub struct DocumentGraph {
    pub(crate) file: FilePersistence,
    pub(crate) chunks: Vec<IndexedChunk>,
}

/// Current, non-deleted file listed for its owner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSummary {
    pub file_id: String,
    pub display_name: String,
    pub mime_type: String,
    pub byte_size: i64,
    pub version_id: String,
    pub version_number: i64,
    #[serde(serialize_with = "crate::shared::serialization::serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "crate::shared::serialization::serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

/// Bytes of a file's current version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredDocumentBlob {
    pub file_id: String,
    pub display_name: String,
    pub mime_type: String,
    pub file_bytes: Vec<u8>,
}
