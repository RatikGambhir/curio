use std::sync::{Arc, Mutex};

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};

use super::*;

#[derive(Clone, Default)]
struct MockProvider {
    batch_sizes: Arc<Mutex<Vec<usize>>>,
    status: Option<StatusCode>,
}

async fn mock_embeddings(
    State(provider): State<MockProvider>,
    Json(request): Json<Value>,
) -> (StatusCode, Json<Value>) {
    if let Some(status) = provider.status {
        return (
            status,
            Json(json!({ "error": { "message": "secret upstream detail" } })),
        );
    }
    let inputs = request["input"].as_array().unwrap();
    provider.batch_sizes.lock().unwrap().push(inputs.len());
    // Return items out of order to prove index-based reassembly.
    let data = inputs
        .iter()
        .enumerate()
        .rev()
        .map(|(index, input)| {
            let length = input.as_str().unwrap().len() as f64;
            json!({ "index": index, "embedding": [length, 0.5] })
        })
        .collect::<Vec<_>>();
    (StatusCode::OK, Json(json!({ "data": data })))
}

async fn start_mock(provider: MockProvider) -> String {
    let app = Router::new()
        .route("/v1/embeddings", post(mock_embeddings))
        .with_state(provider);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{address}")
}

fn client(base_url: String) -> OpenAiDocumentsClient {
    OpenAiDocumentsClient::new(
        "local-test-value".to_owned(),
        base_url,
        "embedding-model".to_owned(),
        "image-model".to_owned(),
    )
}

#[tokio::test]
async fn embeddings_are_batched_and_returned_in_input_order() {
    let provider = MockProvider::default();
    let base_url = start_mock(provider.clone()).await;
    let inputs = (0..130)
        .map(|index| "x".repeat(index + 1))
        .collect::<Vec<_>>();

    let embeddings = client(base_url).embed(inputs).await.unwrap();

    assert_eq!(provider.batch_sizes.lock().unwrap().as_slice(), [64, 64, 2]);
    assert_eq!(embeddings.len(), 130);
    for (index, embedding) in embeddings.iter().enumerate() {
        assert_eq!(embedding, &vec![(index + 1) as f32, 0.5]);
    }
}

#[tokio::test]
async fn provider_errors_are_sanitized() {
    let base_url = start_mock(MockProvider {
        status: Some(StatusCode::TOO_MANY_REQUESTS),
        ..MockProvider::default()
    })
    .await;

    let error = client(base_url)
        .embed(vec!["text".to_owned()])
        .await
        .unwrap_err();

    assert_eq!(error, "the provider is rate limiting requests");
    assert!(!error.contains("secret upstream detail"));
}

#[tokio::test]
async fn blank_inputs_are_rejected_before_calling_the_provider() {
    let client = client("http://127.0.0.1:1".to_owned());

    assert!(client.embed(Vec::new()).await.is_err());
    assert_eq!(
        client
            .embed(vec!["text".to_owned(), "  ".to_owned()])
            .await
            .unwrap_err(),
        "cannot embed empty document content at input index 1"
    );
}

#[test]
fn embedding_responses_must_cover_every_input_exactly_once() {
    assert!(extract_embeddings(&json!({ "data": [] }), 1).is_err());
    assert!(
        extract_embeddings(
            &json!({ "data": [
                { "index": 0, "embedding": [1.0] },
                { "index": 0, "embedding": [2.0] },
            ] }),
            1,
        )
        .is_err()
    );
    assert!(
        extract_embeddings(&json!({ "data": [{ "index": 1, "embedding": [1.0] }] }), 1).is_err()
    );
    assert!(
        extract_embeddings(&json!({ "data": [{ "index": 0, "embedding": ["x"] }] }), 1).is_err()
    );
}

#[test]
fn response_text_is_read_from_output_text_or_output_parts() {
    assert_eq!(
        extract_response_text(&json!({ "output_text": " described " })).as_deref(),
        Some("described")
    );
    assert_eq!(
        extract_response_text(&json!({ "output": [{ "content": [
            { "type": "output_text", "text": "part one " },
            { "type": "refusal", "text": "ignored" },
            { "type": "output_text", "text": "part two" },
        ] }] }))
        .as_deref(),
        Some("part one part two")
    );
    assert_eq!(extract_response_text(&json!({ "output": [] })), None);
}
