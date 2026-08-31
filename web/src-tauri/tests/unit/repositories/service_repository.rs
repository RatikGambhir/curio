use serde_json::Value;

use super::*;

#[test]
fn paths_are_scoped_to_the_configured_origin() {
    let repository = ServiceRepository::new("https://service.example/base?token=secret#fragment")
        .expect("valid service URL");
    let endpoint = repository
        .endpoint("/v1/conversations?limit=10")
        .expect("valid service path");

    assert_eq!(repository.base_url.as_str(), "https://service.example/");
    assert_eq!(
        endpoint.as_str(),
        "https://service.example/v1/conversations?limit=10"
    );
    assert!(repository.endpoint("v1/conversations").is_err());
    assert!(repository
        .endpoint("//attacker.example/v1/conversations")
        .is_err());
    // URL parsers treat backslashes as authority separators for HTTPS URLs.
    // Reject them before same-origin credentials can become an extra
    // Authorization header in reqwest.
    assert!(repository
        .endpoint(r"/\user:secret@service.example/v1/conversations")
        .is_err());
    assert!(repository.endpoint(r"/v1\conversations").is_err());
    assert!(repository.endpoint("/v1/conversations#private").is_err());
    assert!(ServiceRepository::new("file:///tmp/service").is_err());
    assert!(ServiceRepository::new("https://user:secret@service.example").is_err());
}

#[test]
fn requests_forward_methods_payloads_and_bearer_tokens() {
    let repository = ServiceRepository::new("https://service.example").expect("repository");
    let payload = serde_json::json!({ "conversationId": "conversation-1" });
    let post = repository
        .build_request(&ServiceRequest::new(
            ServiceMethod::Post,
            "/v1/chat/stream".to_owned(),
            Some(payload.clone()),
            Some("token-1".to_owned()),
        ))
        .expect("post request");
    let body: Value = serde_json::from_slice(
        post.body()
            .and_then(|body| body.as_bytes())
            .expect("JSON request body"),
    )
    .expect("decode request body");

    assert_eq!(post.method(), Method::POST);
    assert_eq!(
        post.url().as_str(),
        "https://service.example/v1/chat/stream"
    );
    assert_eq!(body, payload);
    assert_eq!(
        post.headers().get(AUTHORIZATION).expect("auth"),
        "Bearer token-1"
    );

    let get = repository
        .build_request(&ServiceRequest::new(
            ServiceMethod::Get,
            "/v1/conversations".to_owned(),
            Some(serde_json::json!({ "limit": 10, "scope": "recent" })),
            None,
        ))
        .expect("get request");
    assert!(get.url().as_str().contains("limit=10"));
    assert!(get.url().as_str().contains("scope=recent"));
    assert!(get.headers().get(AUTHORIZATION).is_none());

    assert_eq!(http_method(ServiceMethod::Get), Method::GET);
    assert_eq!(http_method(ServiceMethod::Patch), Method::PATCH);
    assert_eq!(http_method(ServiceMethod::Delete), Method::DELETE);
}
