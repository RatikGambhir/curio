//! Owner-scoped semantic and keyword search over current document chunks.
use super::model::{KeywordSearchRequest, VectorSearchRequest};
use crate::domains::documents::{
    index::model::{KeywordChunkHit, KeywordSearch, VectorChunkHit, VectorSearch, search_limit},
    model::{DocumentError, Embedder},
};

/// Read-model port for ranked chunk retrieval.
pub trait ChunkSearch: Clone + Send + Sync + 'static {
    fn search_vector(
        &self,
        search: &VectorSearch,
    ) -> impl Future<Output = Result<Vec<VectorChunkHit>, DocumentError>> + Send;

    fn search_keyword(
        &self,
        search: &KeywordSearch,
    ) -> impl Future<Output = Result<Vec<KeywordChunkHit>, DocumentError>> + Send;
}

#[derive(Clone)]
pub struct SearchService<I, E> {
    index: I,
    embedder: E,
}

impl<I: ChunkSearch, E: Embedder> SearchService<I, E> {
    pub fn new(index: I, embedder: E) -> Self {
        Self { index, embedder }
    }

    pub async fn vector(
        &self,
        owner_id: &str,
        request: VectorSearchRequest,
    ) -> Result<Vec<VectorChunkHit>, DocumentError> {
        // Validate the limit before paying for an embedding request.
        search_limit(request.limit)?;
        let embedding = match (request.query_text, request.query_embedding) {
            (Some(text), None) => self.embed_query(text).await?,
            (None, Some(embedding)) => embedding,
            _ => {
                return Err(DocumentError::invalid(
                    "Provide exactly one of queryText or queryEmbedding.",
                ));
            }
        };
        let search = VectorSearch::new(owner_id, embedding, request.limit)?;
        self.index.search_vector(&search).await
    }

    pub async fn keyword(
        &self,
        owner_id: &str,
        request: KeywordSearchRequest,
    ) -> Result<Vec<KeywordChunkHit>, DocumentError> {
        let search = KeywordSearch::new(owner_id, &request.query_text, request.limit)?;
        self.index.search_keyword(&search).await
    }

    async fn embed_query(&self, text: String) -> Result<Vec<f32>, DocumentError> {
        let text = text.trim().to_owned();
        if text.is_empty() {
            return Err(DocumentError::invalid("queryText cannot be empty"));
        }
        self.embedder
            .embed(vec![text])
            .await
            .map_err(DocumentError::Unavailable)?
            .pop()
            .ok_or(DocumentError::Internal)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/domains/documents/search/service.rs"]
mod tests;
