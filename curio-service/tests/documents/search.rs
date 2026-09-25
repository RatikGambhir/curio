//! Vector and keyword search over current, owned, non-deleted chunks.
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};

use crate::support::{OWNER_A, OWNER_B, TestDocuments, UPSTREAM_SECRET, docx_bytes, split_json};

const VECTOR: &str = "/v1/documents/search/vector";
const KEYWORD: &str = "/v1/documents/search/keyword";

/// Two owner-A documents (one about revenue) and one owner-B revenue document.
async fn seeded(test_name: &str) -> Option<(TestDocuments, String, String)> {
    let t = TestDocuments::start(test_name).await?;
    let finance = t
        .ingest(
            OWNER_A,
            "finance.docx",
            &docx_bytes(&["Quarterly revenue grew twelve percent."]),
        )
        .await;
    let office = t
        .ingest(
            OWNER_A,
            "office.docx",
            &docx_bytes(&["The office relocation is scheduled for spring."]),
        )
        .await;
    t.ingest(
        OWNER_B,
        "private.docx",
        &docx_bytes(&["Owner B revenue forecast."]),
    )
    .await;
    Some((t, finance, office))
}

fn file_ids(hits: &Value) -> Vec<&str> {
    hits.as_array()
        .unwrap_or_else(|| panic!("expected hits: {hits}"))
        .iter()
        .map(|hit| hit["fileId"].as_str().unwrap())
        .collect()
}

#[tokio::test]
async fn vector_search_ranks_by_cosine_distance_within_the_owner() {
    let Some((t, finance, office)) =
        seeded("vector_search_ranks_by_cosine_distance_within_the_owner").await
    else {
        return;
    };

    let (status, hits) = t
        .post_json(VECTOR, OWNER_A, json!({ "queryText": "revenue" }))
        .await;
    assert_eq!(status, StatusCode::OK, "{hits}");
    assert_eq!(file_ids(&hits), [finance.as_str(), office.as_str()]);
    let (near, far) = (
        hits[0]["distance"].as_f64().unwrap(),
        hits[1]["distance"].as_f64().unwrap(),
    );
    assert_eq!(near, 0.0, "identical direction, clamped at zero");
    assert!(far > near && far <= 2.0);
    assert_eq!(hits[0]["displayName"], "finance.docx");
    assert_eq!(hits[0]["text"], "Quarterly revenue grew twelve percent.");

    // A precomputed embedding bypasses the provider and ranks the same way.
    let (_, direct) = t
        .post_json(
            VECTOR,
            OWNER_A,
            json!({ "queryEmbedding": [0.1, 1.0], "limit": 1 }),
        )
        .await;
    assert_eq!(file_ids(&direct), [office.as_str()], "limit is respected");

    // Vectors of another dimension never match rather than erroring.
    let (status, mismatched) = t
        .post_json(
            VECTOR,
            OWNER_A,
            json!({ "queryEmbedding": [1.0, 0.0, 0.0] }),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(mismatched, json!([]));

    // A zero vector has no direction, so nothing is comparable.
    let (status, zero) = t
        .post_json(VECTOR, OWNER_A, json!({ "queryEmbedding": [0.0, 0.0] }))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(zero, json!([]));

    t.finish().await;
}

#[tokio::test]
async fn keyword_search_uses_full_text_semantics_within_the_owner() {
    let Some((t, finance, office)) =
        seeded("keyword_search_uses_full_text_semantics_within_the_owner").await
    else {
        return;
    };

    let search = |query: &'static str| {
        let t = &t;
        async move {
            let (status, hits) = t
                .post_json(KEYWORD, OWNER_A, json!({ "queryText": query }))
                .await;
            assert_eq!(status, StatusCode::OK, "{query}: {hits}");
            hits
        }
    };

    let hits = search("revenue").await;
    assert_eq!(
        file_ids(&hits),
        [finance.as_str()],
        "owner B's revenue is hidden"
    );
    assert!(hits[0]["score"].as_f64().unwrap() > 0.0);

    // English stemming matches other word forms.
    assert_eq!(file_ids(&search("relocating").await), [office.as_str()]);
    // Web-search syntax: OR, quoted phrases, and exclusion.
    assert_eq!(file_ids(&search("revenue or relocation").await).len(), 2);
    assert_eq!(
        file_ids(&search("\"revenue grew\"").await),
        [finance.as_str()]
    );
    assert!(file_ids(&search("revenue -quarterly").await).is_empty());
    // Stop words and punctuation-only queries are valid and simply match nothing.
    assert_eq!(search("the and of").await, json!([]));
    assert_eq!(search("&|!:()<->*").await, json!([]));
    assert_eq!(search("nonexistentterm").await, json!([]));

    t.finish().await;
}

#[tokio::test]
async fn search_requests_are_validated() {
    let Some((t, _, _)) = seeded("search_requests_are_validated").await else {
        return;
    };

    let invalid_vector = [
        json!({}),
        json!({ "queryText": "revenue", "queryEmbedding": [1.0, 0.0] }),
        json!({ "queryText": "   " }),
        json!({ "queryEmbedding": [] }),
        json!({ "queryText": "revenue", "limit": 0 }),
        json!({ "queryText": "revenue", "limit": 101 }),
    ];
    for body in invalid_vector {
        let (status, error) = t.post_json(VECTOR, OWNER_A, body.clone()).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}: {error}");
        assert!(error["error"].is_string(), "{body}: {error}");
    }

    let invalid_keyword = [
        json!({ "queryText": "" }),
        json!({ "queryText": "  \t " }),
        json!({ "queryText": "revenue", "limit": 0 }),
        json!({ "queryText": "revenue", "limit": 101 }),
    ];
    for body in invalid_keyword {
        let (status, _) = t.post_json(KEYWORD, OWNER_A, body.clone()).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    }

    // Shape errors are framework JSON rejections. A client-supplied owner is an
    // unknown field, so it can never widen the search scope.
    for (path, body) in [
        (KEYWORD, json!({})),
        (
            KEYWORD,
            json!({ "queryText": "revenue", "workspaceId": OWNER_B }),
        ),
        (VECTOR, json!({ "queryText": "revenue", "userId": OWNER_B })),
        (VECTOR, json!({ "queryEmbedding": ["x"] })),
        (KEYWORD, json!({ "queryText": "revenue", "limit": -1 })),
    ] {
        let (status, _) = t.post_json(path, OWNER_A, body.clone()).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{path} {body}");
    }

    let malformed = t
        .send(
            Request::post(KEYWORD)
                .header(header::AUTHORIZATION, format!("Bearer {OWNER_A}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{not json"))
                .unwrap(),
        )
        .await;
    assert_eq!(malformed.status(), StatusCode::BAD_REQUEST);
    let untyped = t
        .send(
            Request::post(KEYWORD)
                .header(header::AUTHORIZATION, format!("Bearer {OWNER_A}"))
                .body(Body::from(json!({ "queryText": "revenue" }).to_string()))
                .unwrap(),
        )
        .await;
    assert_eq!(untyped.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    // The limit boundaries themselves are accepted, and the default is 10.
    for (path, body) in [
        (KEYWORD, json!({ "queryText": "revenue", "limit": 100 })),
        (VECTOR, json!({ "queryText": "revenue", "limit": 1 })),
        (VECTOR, json!({ "queryText": "revenue" })),
    ] {
        assert_eq!(
            t.post_json(path, OWNER_A, body.clone()).await.0,
            StatusCode::OK,
            "{body}"
        );
    }

    t.finish().await;
}

#[tokio::test]
async fn the_default_limit_caps_results_at_ten() {
    let Some(t) = TestDocuments::start("the_default_limit_caps_results_at_ten").await else {
        return;
    };
    for index in 0..12 {
        t.ingest(
            OWNER_A,
            &format!("revenue-{index:02}.docx"),
            &docx_bytes(&[&format!("Revenue note number {index}.")]),
        )
        .await;
    }

    for path in [VECTOR, KEYWORD] {
        let (_, hits) = t
            .post_json(path, OWNER_A, json!({ "queryText": "revenue" }))
            .await;
        assert_eq!(hits.as_array().unwrap().len(), 10, "{path}");
        let (_, all) = t
            .post_json(
                path,
                OWNER_A,
                json!({ "queryText": "revenue", "limit": 50 }),
            )
            .await;
        assert_eq!(all.as_array().unwrap().len(), 12, "{path}");
    }

    t.finish().await;
}

#[tokio::test]
async fn provider_outages_fail_text_vector_search_but_not_other_search() {
    let Some((t, finance, _)) =
        seeded("provider_outages_fail_text_vector_search_but_not_other_search").await
    else {
        return;
    };
    t.openai.fail_with(Some(StatusCode::BAD_GATEWAY));

    let (status, body) = t
        .post_json(VECTOR, OWNER_A, json!({ "queryText": "revenue" }))
        .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"], "the provider is temporarily unavailable");
    assert!(!body.to_string().contains(UPSTREAM_SECRET));

    // Neither keyword search nor precomputed embeddings need the provider.
    let (_, keyword) = t
        .post_json(KEYWORD, OWNER_A, json!({ "queryText": "revenue" }))
        .await;
    assert_eq!(file_ids(&keyword), [finance.as_str()]);
    let (status, _) = t
        .post_json(VECTOR, OWNER_A, json!({ "queryEmbedding": [1.0, 0.1] }))
        .await;
    assert_eq!(status, StatusCode::OK);

    t.finish().await;
}

#[tokio::test]
async fn an_owner_with_no_documents_gets_empty_results() {
    let Some(t) = TestDocuments::start("an_owner_with_no_documents_gets_empty_results").await
    else {
        return;
    };

    for (path, body) in [
        (VECTOR, json!({ "queryText": "revenue" })),
        (KEYWORD, json!({ "queryText": "revenue" })),
    ] {
        let request = Request::post(path)
            .header(header::AUTHORIZATION, format!("Bearer {OWNER_A}"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        assert_eq!(
            split_json(t.send(request).await).await,
            (StatusCode::OK, json!([])),
            "{path}"
        );
    }

    t.finish().await;
}
