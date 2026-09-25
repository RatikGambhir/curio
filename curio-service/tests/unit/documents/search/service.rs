use std::sync::{Arc, Mutex};

use super::*;
use crate::documents::ingestion::test_support::FixedEmbedder;

#[derive(Clone, Default)]
struct RecordingIndex {
    vector: Arc<Mutex<Vec<VectorSearch>>>,
    keyword: Arc<Mutex<Vec<KeywordSearch>>>,
}

impl ChunkSearch for RecordingIndex {
    async fn search_vector(
        &self,
        search: &VectorSearch,
    ) -> Result<Vec<VectorChunkHit>, DocumentError> {
        self.vector.lock().unwrap().push(search.clone());
        Ok(Vec::new())
    }

    async fn search_keyword(
        &self,
        search: &KeywordSearch,
    ) -> Result<Vec<KeywordChunkHit>, DocumentError> {
        self.keyword.lock().unwrap().push(search.clone());
        Ok(Vec::new())
    }
}

fn vector_request(
    query_text: Option<&str>,
    query_embedding: Option<Vec<f32>>,
    limit: usize,
) -> VectorSearchRequest {
    VectorSearchRequest {
        query_text: query_text.map(str::to_owned),
        query_embedding,
        limit,
    }
}

#[tokio::test]
async fn query_text_is_embedded_server_side_and_scoped_to_the_owner() {
    let index = RecordingIndex::default();
    let embedder = FixedEmbedder::default();
    let search = SearchService::new(index.clone(), embedder.clone());

    search
        .vector("user-a", vector_request(Some("  revenue  "), None, 5))
        .await
        .unwrap();

    let recorded = index.vector.lock().unwrap();
    assert_eq!(recorded[0].owner_id(), "user-a");
    assert_eq!(recorded[0].embedding(), [7.0, 1.0]);
    assert_eq!(recorded[0].limit(), 5);
    assert_eq!(
        embedder.calls.lock().unwrap().as_slice(),
        [vec!["revenue".to_string()]]
    );
}

#[tokio::test]
async fn vector_search_requires_exactly_one_query_form() {
    let search = SearchService::new(RecordingIndex::default(), FixedEmbedder::default());

    for request in [
        vector_request(None, None, 5),
        vector_request(Some("revenue"), Some(vec![1.0]), 5),
    ] {
        assert!(matches!(
            search.vector("user-a", request).await,
            Err(DocumentError::Invalid(_))
        ));
    }
}

#[tokio::test]
async fn invalid_limits_and_embeddings_are_rejected_before_provider_calls() {
    let embedder = FixedEmbedder::default();
    let search = SearchService::new(RecordingIndex::default(), embedder.clone());

    for request in [
        vector_request(Some("revenue"), None, 0),
        vector_request(Some("revenue"), None, 101),
        vector_request(None, Some(Vec::new()), 5),
        vector_request(None, Some(vec![f32::INFINITY]), 5),
    ] {
        assert!(matches!(
            search.vector("user-a", request).await,
            Err(DocumentError::Invalid(_))
        ));
    }
    assert!(embedder.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn provider_failures_are_unavailable_errors() {
    let search = SearchService::new(
        RecordingIndex::default(),
        FixedEmbedder {
            error: Some("the provider is rate limiting requests"),
            ..FixedEmbedder::default()
        },
    );

    assert_eq!(
        search
            .vector("user-a", vector_request(Some("revenue"), None, 5))
            .await,
        Err(DocumentError::Unavailable(
            "the provider is rate limiting requests".to_owned()
        ))
    );
}

#[tokio::test]
async fn keyword_search_trims_the_query_and_validates_it() {
    let index = RecordingIndex::default();
    let search = SearchService::new(index.clone(), FixedEmbedder::default());

    search
        .keyword(
            "user-a",
            KeywordSearchRequest {
                query_text: "  revenue growth ".to_owned(),
                limit: 3,
            },
        )
        .await
        .unwrap();
    assert_eq!(index.keyword.lock().unwrap()[0].text(), "revenue growth");

    assert!(matches!(
        search
            .keyword(
                "user-a",
                KeywordSearchRequest {
                    query_text: "   ".to_owned(),
                    limit: 3,
                },
            )
            .await,
        Err(DocumentError::Invalid(_))
    ));
}

#[test]
fn search_requests_reject_unknown_fields_such_as_a_client_owner() {
    assert!(
        serde_json::from_value::<KeywordSearchRequest>(serde_json::json!({
            "queryText": "revenue",
            "workspaceId": "user-b",
        }))
        .is_err()
    );
    let defaulted: KeywordSearchRequest =
        serde_json::from_value(serde_json::json!({ "queryText": "revenue" })).unwrap();
    assert_eq!(defaulted.limit, 10);
}
