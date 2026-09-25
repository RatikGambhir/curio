//! Chunk index values: the rows written per file version and the owner-scoped
//! read model used by chunk listing and vector/keyword search.
use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::documents::domain::DocumentError;

pub const MAX_CHUNK_SEARCH_LIMIT: usize = 100;

/// A chunk row written for exactly one file version.
#[derive(Debug, Clone, PartialEq)]
pub struct IndexedChunk {
    pub chunk_id: String,
    pub owner_id: String,
    pub file_id: String,
    pub version_id: String,
    pub chunk_index: i64,
    pub text: String,
    pub embedding: Vec<f32>,
    pub chunk_sha256: String,
    pub token_count: i64,
    pub page_start: Option<i64>,
    pub page_end: Option<i64>,
    pub char_start: i64,
    pub char_end: i64,
    pub section_path: String,
}

/// Checks that every chunk belongs to one version and is internally valid.
pub fn validate_chunk_rows(
    owner_id: &str,
    file_id: &str,
    version_id: &str,
    chunks: &[IndexedChunk],
) -> Result<(), String> {
    let mut chunk_ids = HashSet::with_capacity(chunks.len());
    let mut chunk_indices = HashSet::with_capacity(chunks.len());
    let mut embedding_dimension = None;
    for chunk in chunks {
        if chunk.owner_id != owner_id || chunk.file_id != file_id || chunk.version_id != version_id
        {
            return Err(format!(
                "chunk `{}` identity does not match its file version",
                chunk.chunk_id
            ));
        }
        if chunk.chunk_id.trim().is_empty() || chunk.chunk_sha256.trim().is_empty() {
            return Err("chunk identity fields cannot be empty".to_string());
        }
        if !chunk_ids.insert(&chunk.chunk_id) || !chunk_indices.insert(chunk.chunk_index) {
            return Err(format!(
                "chunk `{}` duplicates a chunk identity or index",
                chunk.chunk_id
            ));
        }
        if chunk.chunk_index < 0
            || chunk.token_count < 0
            || chunk.char_start < 0
            || chunk.char_end < chunk.char_start
        {
            return Err(format!(
                "chunk `{}` has an invalid numeric range",
                chunk.chunk_id
            ));
        }
        if chunk.page_start.is_some() != chunk.page_end.is_some()
            || chunk
                .page_start
                .zip(chunk.page_end)
                .is_some_and(|(start, end)| start > end)
        {
            return Err(format!(
                "chunk `{}` has an invalid page range",
                chunk.chunk_id
            ));
        }
        if chunk.embedding.is_empty() || chunk.embedding.iter().any(|value| !value.is_finite()) {
            return Err(format!(
                "chunk `{}` has an invalid embedding",
                chunk.chunk_id
            ));
        }
        match embedding_dimension {
            Some(expected) if expected != chunk.embedding.len() => {
                return Err(format!(
                    "chunk `{}` embedding dimension does not match the version",
                    chunk.chunk_id
                ));
            }
            None => embedding_dimension = Some(chunk.embedding.len()),
            _ => {}
        }
    }
    Ok(())
}

/// A chunk of a current, non-deleted file version.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkRecord {
    pub chunk_id: String,
    pub file_id: String,
    pub version_id: String,
    pub display_name: String,
    pub chunk_index: i64,
    pub text: String,
    pub chunk_sha256: String,
    pub token_count: i64,
    pub page_start: Option<i64>,
    pub page_end: Option<i64>,
    pub char_start: i64,
    pub char_end: i64,
    pub section_path: String,
    #[serde(serialize_with = "crate::serialization::serialize_timestamp")]
    pub created_at: DateTime<Utc>,
}

/// Cosine distance: `0` is identical direction, `2` is opposite.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VectorChunkHit {
    #[serde(flatten)]
    pub chunk: ChunkRecord,
    pub distance: f64,
}

/// Full-text relevance; higher is better.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeywordChunkHit {
    #[serde(flatten)]
    pub chunk: ChunkRecord,
    pub score: f64,
}

/// Validated owner-scoped nearest-neighbour query.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorSearch {
    owner_id: String,
    embedding: Vec<f32>,
    limit: i64,
}

impl VectorSearch {
    pub fn new(owner_id: &str, embedding: Vec<f32>, limit: usize) -> Result<Self, DocumentError> {
        if embedding.is_empty() {
            return Err(DocumentError::invalid("queryEmbedding cannot be empty"));
        }
        if embedding.iter().any(|value| !value.is_finite()) {
            return Err(DocumentError::invalid(
                "queryEmbedding must contain only finite values",
            ));
        }
        Ok(Self {
            owner_id: required_owner(owner_id)?,
            embedding,
            limit: search_limit(limit)?,
        })
    }

    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    pub fn embedding(&self) -> &[f32] {
        &self.embedding
    }

    pub fn limit(&self) -> i64 {
        self.limit
    }
}

/// Validated owner-scoped full-text query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordSearch {
    owner_id: String,
    text: String,
    limit: i64,
}

impl KeywordSearch {
    pub fn new(owner_id: &str, text: &str, limit: usize) -> Result<Self, DocumentError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(DocumentError::invalid("queryText cannot be empty"));
        }
        Ok(Self {
            owner_id: required_owner(owner_id)?,
            text: text.to_owned(),
            limit: search_limit(limit)?,
        })
    }

    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn limit(&self) -> i64 {
        self.limit
    }
}

fn required_owner(owner_id: &str) -> Result<String, DocumentError> {
    let owner_id = owner_id.trim();
    if owner_id.is_empty() {
        return Err(DocumentError::invalid("A document owner is required."));
    }
    Ok(owner_id.to_owned())
}

pub fn search_limit(limit: usize) -> Result<i64, DocumentError> {
    if limit == 0 {
        return Err(DocumentError::invalid("limit must be greater than zero"));
    }
    if limit > MAX_CHUNK_SEARCH_LIMIT {
        return Err(DocumentError::invalid(format!(
            "limit must not exceed {MAX_CHUNK_SEARCH_LIMIT}"
        )));
    }
    i64::try_from(limit).map_err(|_| DocumentError::invalid("limit is out of range"))
}
