//! Authentication and owner isolation across every documents route.
use axum::{
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use serde_json::json;

use crate::support::{OWNER_A, OWNER_B, TestDocuments, docx_bytes, split_json};

fn routes(file_id: &str) -> Vec<(Method, String)> {
    vec![
        (Method::POST, "/v1/documents/process".to_owned()),
        (Method::POST, "/v1/documents/jobs".to_owned()),
        (Method::GET, "/v1/documents/jobs/some-job/events".to_owned()),
        (Method::GET, "/v1/documents".to_owned()),
        (Method::DELETE, format!("/v1/documents/{file_id}")),
        (Method::GET, format!("/v1/documents/{file_id}/chunks")),
        (Method::GET, format!("/v1/documents/{file_id}/text")),
        (Method::GET, format!("/v1/documents/{file_id}/pdf")),
        (Method::POST, "/v1/documents/search/vector".to_owned()),
        (Method::POST, "/v1/documents/search/keyword".to_owned()),
    ]
}

#[tokio::test]
async fn every_route_requires_a_well_formed_bearer_token() {
    let Some(t) = TestDocuments::start("every_route_requires_a_well_formed_bearer_token").await
    else {
        return;
    };
    let file_id = t
        .ingest(OWNER_A, "memo.docx", &docx_bytes(&["Protected body."]))
        .await;

    for (method, path) in routes(&file_id) {
        for authorization in [
            None,
            Some("Bearer ".to_owned()),
            Some(format!("Basic {OWNER_A}")),
            Some(format!("bearer {OWNER_A}")),
            Some(format!("Bearer {OWNER_A} extra")),
        ] {
            let mut request = Request::builder().method(method.clone()).uri(&path);
            if let Some(value) = &authorization {
                request = request.header(header::AUTHORIZATION, value);
            }
            let response = t
                .send(
                    request
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(json!({ "queryText": "x" }).to_string()))
                        .unwrap(),
                )
                .await;
            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "{method} {path} with {authorization:?}"
            );
        }
    }

    // The unauthenticated attempts changed nothing.
    assert_eq!(t.listed_names(OWNER_A).await, ["memo.docx"]);

    t.finish().await;
}

#[tokio::test]
async fn another_owner_cannot_see_or_change_a_document() {
    let Some(t) = TestDocuments::start("another_owner_cannot_see_or_change_a_document").await
    else {
        return;
    };
    let file_id = t
        .ingest(
            OWNER_A,
            "secret.docx",
            &docx_bytes(&["Confidential revenue plan."]),
        )
        .await;

    for suffix in ["text", "chunks", "pdf"] {
        let path = format!("/v1/documents/{file_id}/{suffix}");
        let (foreign, foreign_body) = t.get_json(&path, OWNER_B).await;
        let (missing, missing_body) = t
            .get_json(&format!("/v1/documents/missing-file/{suffix}"), OWNER_B)
            .await;
        assert_eq!(foreign, StatusCode::NOT_FOUND, "{suffix}");
        assert_eq!(
            (foreign, foreign_body),
            (missing, missing_body),
            "{suffix}: a foreign file is indistinguishable from a missing one"
        );
    }
    assert_eq!(
        t.delete(&format!("/v1/documents/{file_id}"), OWNER_B)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    assert!(t.listed_names(OWNER_B).await.is_empty());
    for path in [
        "/v1/documents/search/keyword",
        "/v1/documents/search/vector",
    ] {
        let (status, hits) = t
            .post_json(
                path,
                OWNER_B,
                json!({ "queryText": "confidential revenue" }),
            )
            .await;
        assert_eq!((status, hits), (StatusCode::OK, json!([])), "{path}");
    }

    // The owner's document survived the foreign delete attempt.
    let (status, text) = t
        .get_json(&format!("/v1/documents/{file_id}/text"), OWNER_A)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(text["text"], "Confidential revenue plan.");

    t.finish().await;
}

#[tokio::test]
async fn cors_preflight_allows_the_configured_origin_for_document_methods() {
    let Some(t) =
        TestDocuments::start("cors_preflight_allows_the_configured_origin_for_document_methods")
            .await
    else {
        return;
    };

    for method in ["POST", "DELETE"] {
        let response = t
            .send(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/v1/documents/process")
                    .header(header::ORIGIN, "http://localhost:5173")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, method)
                    .header(
                        header::ACCESS_CONTROL_REQUEST_HEADERS,
                        "authorization,content-type",
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;
        assert_eq!(
            response.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
            "http://localhost:5173",
            "{method}"
        );
        let (status, _) = split_json(response).await;
        assert!(status.is_success(), "{method}: {status}");
    }

    t.finish().await;
}
