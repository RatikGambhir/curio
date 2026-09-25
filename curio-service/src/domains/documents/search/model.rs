//! Search request values. The owner always comes from the authenticated
//! identity, never from the request body.
use serde::Deserialize;

pub const DEFAULT_SEARCH_LIMIT: usize = 10;

fn default_limit() -> usize {
    DEFAULT_SEARCH_LIMIT
}

/// Exactly one of `queryText` (embedded server-side) or a precomputed
/// `queryEmbedding` must be supplied.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VectorSearchRequest {
    pub query_text: Option<String>,
    pub query_embedding: Option<Vec<f32>>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeywordSearchRequest {
    pub query_text: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}
