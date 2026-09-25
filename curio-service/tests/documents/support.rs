//! Shared harness for the documents integration suite: a full Curio router on
//! a disposable PostgreSQL schema, a controllable mock OpenAI embeddings
//! server, and builders for PDF, DOCX, and multipart request bodies.
use std::{
    io::Cursor,
    sync::{Arc, Mutex},
};

use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode, header},
    response::Response,
    routing::post,
};
use curio_service::{
    app::config::{DocumentsConfig, ServiceConfig},
    app_with_database,
};
use docx_rust::{Docx, document::Paragraph};
use http_body_util::BodyExt;
use lopdf::{
    Document, Object, Stream,
    content::{Content, Operation},
    dictionary,
};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::postgres::PostgresFixture;

pub const OWNER_A: &str = "user-a";
pub const OWNER_B: &str = "user-b";
/// Authenticates but has no stored profile.
pub const OWNER_WITHOUT_PROFILE: &str = "user-without-profile";
/// Text the mock provider puts in error bodies; it must never reach clients.
pub const UPSTREAM_SECRET: &str = "upstream-secret-detail";
pub const DOCX_MIME: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document";

const BOUNDARY: &str = "curio-documents-boundary";

#[derive(Default)]
struct MockState {
    failure: Option<StatusCode>,
    batch_sizes: Vec<usize>,
    models: Vec<String>,
    requests_without_bearer: usize,
}

/// Embeds text mentioning "revenue" toward `[1, 0.1]` and everything else
/// toward `[0.1, 1]`, so vector ranking is deterministic.
#[derive(Clone, Default)]
pub struct MockOpenAi {
    state: Arc<Mutex<MockState>>,
}

impl MockOpenAi {
    pub fn fail_with(&self, status: Option<StatusCode>) {
        self.state.lock().unwrap().failure = status;
    }

    pub fn batch_sizes(&self) -> Vec<usize> {
        self.state.lock().unwrap().batch_sizes.clone()
    }

    pub fn models(&self) -> Vec<String> {
        self.state.lock().unwrap().models.clone()
    }

    pub fn requests_without_bearer(&self) -> usize {
        self.state.lock().unwrap().requests_without_bearer
    }
}

async fn mock_embeddings(
    State(mock): State<MockOpenAi>,
    headers: HeaderMap,
    Json(request): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let mut state = mock.state.lock().unwrap();
    if !headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("Bearer "))
    {
        state.requests_without_bearer += 1;
    }
    if let Some(status) = state.failure {
        return (
            status,
            Json(json!({ "error": { "message": UPSTREAM_SECRET } })),
        );
    }
    let inputs = request["input"].as_array().cloned().unwrap_or_default();
    state.batch_sizes.push(inputs.len());
    state
        .models
        .push(request["model"].as_str().unwrap_or_default().to_owned());
    let data = inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            let text = input.as_str().unwrap_or_default().to_ascii_lowercase();
            let embedding = if text.contains("revenue") {
                [1.0, 0.1]
            } else {
                [0.1, 1.0]
            };
            json!({ "index": index, "embedding": embedding })
        })
        .collect::<Vec<_>>();
    (StatusCode::OK, Json(json!({ "data": data })))
}

pub struct TestDocuments {
    pub app: Router,
    pub openai: MockOpenAi,
    postgres: PostgresFixture,
}

impl TestDocuments {
    /// Returns `None` (and the caller returns early) without `CURIO_TEST_DATABASE_URL`.
    pub async fn start(test_name: &str) -> Option<Self> {
        Self::start_with(test_name, DocumentsConfig::default()).await
    }

    pub async fn start_with(test_name: &str, documents: DocumentsConfig) -> Option<Self> {
        let postgres = PostgresFixture::provision(test_name).await?;
        let openai = MockOpenAi::default();
        let mock = Router::new()
            .route("/v1/embeddings", post(mock_embeddings))
            .with_state(openai.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let openai_base_url = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move {
            axum::serve(listener, mock).await.unwrap();
        });

        let app = app_with_database(
            ServiceConfig {
                openai_api_key: "local-test-value".to_owned(),
                openai_model: "configured-test-value".to_owned(),
                openai_base_url,
                database_url: postgres.database_url().to_owned(),
                database_schema: postgres.schema().to_owned(),
                database_max_connections: 5,
                database_acquire_timeout_seconds: 5,
                cors_allowed_origins: vec!["http://localhost:5173".to_owned()],
                documents,
            },
            postgres.database().clone(),
        );
        let harness = Self {
            app,
            openai,
            postgres,
        };

        for owner in [OWNER_A, OWNER_B] {
            let (status, _) = harness
                .post_json(
                    "/v1/users",
                    owner,
                    json!({ "id": owner, "name": owner, "email": format!("{owner}@example.com") }),
                )
                .await;
            assert_eq!(status, StatusCode::OK, "seeding profile {owner}");
        }
        Some(harness)
    }

    pub async fn finish(self) {
        self.postgres.cleanup().await;
    }

    pub async fn send(&self, request: Request<Body>) -> Response {
        self.app.clone().oneshot(request).await.unwrap()
    }

    pub async fn get(&self, path: &str, bearer: &str) -> Response {
        self.send(
            authorized(Request::get(path), bearer)
                .body(Body::empty())
                .unwrap(),
        )
        .await
    }

    pub async fn get_json(&self, path: &str, bearer: &str) -> (StatusCode, Value) {
        split_json(self.get(path, bearer).await).await
    }

    pub async fn post_json(&self, path: &str, bearer: &str, body: Value) -> (StatusCode, Value) {
        let request = authorized(Request::post(path), bearer)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        split_json(self.send(request).await).await
    }

    pub async fn delete(&self, path: &str, bearer: &str) -> (StatusCode, Value) {
        let request = authorized(Request::delete(path), bearer)
            .body(Body::empty())
            .unwrap();
        split_json(self.send(request).await).await
    }

    pub async fn upload(
        &self,
        path: &str,
        bearer: &str,
        parts: &[Part<'_>],
    ) -> (StatusCode, Value) {
        split_json(
            self.send(upload_request(path, bearer, multipart(parts)))
                .await,
        )
        .await
    }

    /// Uploads one file through the batch route and returns its per-file result.
    pub async fn process_one(&self, bearer: &str, filename: &str, bytes: &[u8]) -> Value {
        let (status, body) = self
            .upload("/v1/documents/process", bearer, &[file(filename, bytes)])
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body["total"], 1, "{body}");
        body["documents"][0].clone()
    }

    /// Uploads a file that must be ingested (not skipped) and returns its `fileId`.
    pub async fn ingest(&self, bearer: &str, filename: &str, bytes: &[u8]) -> String {
        let document = self.process_one(bearer, filename, bytes).await;
        assert_eq!(document["success"], true, "{document}");
        assert_eq!(document["skipped"], false, "{document}");
        document["fileId"].as_str().unwrap().to_owned()
    }

    pub async fn listed_names(&self, bearer: &str) -> Vec<String> {
        let (status, body) = self.get_json("/v1/documents", bearer).await;
        assert_eq!(status, StatusCode::OK);
        body["documents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|document| document["displayName"].as_str().unwrap().to_owned())
            .collect()
    }
}

pub struct Part<'a> {
    name: &'a str,
    filename: Option<&'a str>,
    bytes: &'a [u8],
}

/// A file in the `files` field.
pub fn file<'a>(filename: &'a str, bytes: &'a [u8]) -> Part<'a> {
    Part {
        name: "files",
        filename: Some(filename),
        bytes,
    }
}

/// A plain text field.
pub fn field<'a>(name: &'a str, value: &'a [u8]) -> Part<'a> {
    Part {
        name,
        filename: None,
        bytes: value,
    }
}

/// A file part in an arbitrary field.
pub fn named_file<'a>(name: &'a str, filename: &'a str, bytes: &'a [u8]) -> Part<'a> {
    Part {
        name,
        filename: Some(filename),
        bytes,
    }
}

pub fn multipart(parts: &[Part<'_>]) -> Vec<u8> {
    let mut body = Vec::new();
    for part in parts {
        body.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
        let disposition = match part.filename {
            Some(filename) => format!(
                "Content-Disposition: form-data; name=\"{}\"; filename=\"{filename}\"\r\n\
                 Content-Type: application/octet-stream\r\n\r\n",
                part.name
            ),
            None => format!(
                "Content-Disposition: form-data; name=\"{}\"\r\n\r\n",
                part.name
            ),
        };
        body.extend_from_slice(disposition.as_bytes());
        body.extend_from_slice(part.bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());
    body
}

pub fn upload_request(path: &str, bearer: &str, body: Vec<u8>) -> Request<Body> {
    authorized(Request::post(path), bearer)
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={BOUNDARY}"),
        )
        .body(Body::from(body))
        .unwrap()
}

fn authorized(builder: axum::http::request::Builder, bearer: &str) -> axum::http::request::Builder {
    builder.header(header::AUTHORIZATION, format!("Bearer {bearer}"))
}

pub async fn body_bytes(response: Response) -> Vec<u8> {
    response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec()
}

/// Splits a response into status and JSON body (`Null` for an empty body).
pub async fn split_json(response: Response) -> (StatusCode, Value) {
    let status = response.status();
    let bytes = body_bytes(response).await;
    if bytes.is_empty() {
        return (status, Value::Null);
    }
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
    (status, body)
}

pub fn docx_bytes(paragraphs: &[&str]) -> Vec<u8> {
    let mut docx = Docx::default();
    for paragraph in paragraphs {
        docx.document
            .push(Paragraph::default().push_text(*paragraph));
    }
    docx.write(Cursor::new(Vec::new())).unwrap().into_inner()
}

/// Builds a PDF with one text line per page; an empty string is a blank page.
pub fn pdf_bytes(pages: &[&str]) -> Vec<u8> {
    let mut document = Document::with_version("1.5");
    let pages_id = document.new_object_id();
    let font_id = document.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Courier",
    });
    let resources_id = document.add_object(dictionary! {
        "Font" => dictionary! { "F1" => font_id },
    });
    let mut kids = Vec::new();
    for text in pages {
        let mut operations = vec![Operation::new("BT", vec![])];
        if !text.is_empty() {
            operations.push(Operation::new("Tf", vec!["F1".into(), 12.into()]));
            operations.push(Operation::new("Td", vec![72.into(), 720.into()]));
            operations.push(Operation::new("Tj", vec![Object::string_literal(*text)]));
        }
        operations.push(Operation::new("ET", vec![]));
        let content_id = document.add_object(Stream::new(
            dictionary! {},
            Content { operations }.encode().unwrap(),
        ));
        let page_id = document.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
        });
        kids.push(page_id.into());
    }
    document.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => kids,
            "Count" => pages.len() as i64,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        }),
    );
    let catalog_id = document.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    document.trailer.set("Root", catalog_id);
    let mut bytes = Vec::new();
    document.save_to(&mut bytes).unwrap();
    bytes
}

/// Repeats a sentence until the text spans at least `minimum_chunks` chunks.
pub fn long_text(sentence: &str, minimum_chunks: usize) -> String {
    // Roughly 4.5 KB of prose fills one 800-token chunk; overshoot generously.
    sentence.repeat(minimum_chunks * 6_000 / sentence.len().max(1))
}

/// Asserts the millisecond UTC wire timestamp shape, e.g. `2026-09-25T10:00:00.000Z`.
pub fn assert_wire_timestamp(value: &Value) {
    let text = value
        .as_str()
        .unwrap_or_else(|| panic!("not a string: {value}"));
    assert_eq!(text.len(), 24, "{text}");
    assert!(text.ends_with('Z') && text.as_bytes()[19] == b'.', "{text}");
    chrono::DateTime::parse_from_rfc3339(text).unwrap();
}

/// Checks that every chunk's byte range selects exactly its text within `document`.
pub fn assert_chunks_index_into(document: &str, chunks: &[Value]) {
    let mut expected_start = 0;
    for (position, chunk) in chunks.iter().enumerate() {
        assert_eq!(chunk["chunkIndex"], position as i64 + 1, "{chunk}");
        let start = chunk["charStart"].as_u64().unwrap() as usize;
        let end = chunk["charEnd"].as_u64().unwrap() as usize;
        assert_eq!(start, expected_start, "chunks must be contiguous");
        assert_eq!(&document[start..end], chunk["text"].as_str().unwrap());
        assert!(chunk["tokenCount"].as_i64().unwrap() <= 800);
        expected_start = end;
    }
    assert_eq!(expected_start, document.len(), "chunks must cover the text");
}
