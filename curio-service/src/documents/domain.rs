//! Documents shared kernel: parsed-document values, content identity, file
//! policy, domain errors, and provider ports. No Axum or SQLx dependencies.
use std::{future::Future, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Largest single upload accepted by ingestion.
pub const MAX_FILE_BYTES: usize = 50 * 1024 * 1024;
/// Largest combined upload accepted in one ingestion request.
pub const MAX_TOTAL_REQUEST_FILE_BYTES: usize = 50 * 1024 * 1024;

/// A parsed source document before persistence.
///
/// `document_id` is derived from the owner and file content, so identical bytes
/// uploaded by the same owner always produce the same document identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub file_id: String,
    pub document_id: String,
    pub owner_id: String,
    pub file_name: String,
    pub source_type: String,
    pub local_path: Option<String>,
    pub file_size_bytes: u64,
    pub token_count: u64,
    pub content_hash: String,
    pub rendered_pdf_path: Option<String>,
}

/// One token-bounded slice of a document's canonical text.
///
/// Offsets are exclusive UTF-8 byte offsets into the document-wide text, and
/// sequence numbers are one-based across the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub chunk_id: String,
    pub document_id: String,
    pub owner_id: String,
    pub text: String,
    pub embedding: Option<Vec<f32>>,
    pub sequence_number: u32,
    pub page_numbers: Option<Vec<u32>>,
    pub start_offset: usize,
    pub end_offset: usize,
    pub token_count: u32,
    pub content_hash: String,
    pub section_title: Option<String>,
}

/// Domain failures shared by every documents use case. Messages are safe to
/// return to clients; storage and provider details are logged, not carried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentError {
    /// The request itself is malformed (for example an unreadable multipart body).
    BadRequest(String),
    /// The request is well formed but violates a documents rule.
    Invalid(String),
    PayloadTooLarge(String),
    NotFound(String),
    Conflict(String),
    UnknownOwner,
    Unavailable(String),
    Internal,
}

impl DocumentError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    /// Client-safe description, used where a failure is reported per document
    /// inside a successful response rather than as an HTTP error.
    pub fn public_message(&self) -> String {
        match self {
            Self::BadRequest(message)
            | Self::Invalid(message)
            | Self::PayloadTooLarge(message)
            | Self::NotFound(message)
            | Self::Conflict(message)
            | Self::Unavailable(message) => message.clone(),
            Self::UnknownOwner => UNKNOWN_OWNER_MESSAGE.to_owned(),
            Self::Internal => INTERNAL_MESSAGE.to_owned(),
        }
    }
}

pub const UNKNOWN_OWNER_MESSAGE: &str = "No profile exists for that user yet.";
pub const INTERNAL_MESSAGE: &str = "The document service is unavailable.";

/// Provider port that turns chunk or query text into embedding vectors,
/// preserving input order. Errors are already sanitized for clients.
pub trait Embedder: Clone + Send + Sync + 'static {
    fn embed(
        &self,
        inputs: Vec<String>,
    ) -> impl Future<Output = Result<Vec<Vec<f32>>, String>> + Send;
}

/// Provider port that produces a retrieval-oriented description of an image.
pub trait ImageDescriber: Send + Sync {
    fn describe_image(
        &self,
        prompt: &str,
        mime_type: &str,
        data_base64: &str,
    ) -> impl Future<Output = Result<String, String>> + Send;
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn document_id_from_content(owner_id: &str, content_hash: &str) -> String {
    sha256_hex(format!("{owner_id}\0{content_hash}").as_bytes())
}

pub fn file_version_id(file_id: &str, content_hash: &str) -> String {
    sha256_hex(format!("{file_id}\0{content_hash}").as_bytes())
}

pub fn infer_supported_mime_type(path: &Path) -> Option<&'static str> {
    match lowercase_extension(path).as_deref() {
        Some("pdf") => Some("application/pdf"),
        Some("txt") => Some("text/plain"),
        Some("md") => Some("text/markdown"),
        Some("json") => Some("application/json"),
        Some("html") => Some("text/html"),
        Some("csv") => Some("text/csv"),
        Some("doc") => Some("application/msword"),
        Some("docx") => Some(DOCX_MIME_TYPE),
        Some("pptx") => {
            Some("application/vnd.openxmlformats-officedocument.presentationml.presentation")
        }
        Some("xls") => Some("application/vnd.ms-excel"),
        Some("xlsx") => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        _ => None,
    }
}

pub fn office_extension_for_mime_type(mime_type: &str) -> Option<&'static str> {
    match mime_type {
        "application/msword" => Some("doc"),
        DOCX_MIME_TYPE => Some("docx"),
        "application/vnd.ms-excel" => Some("xls"),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => Some("xlsx"),
        "application/vnd.ms-powerpoint" => Some("ppt"),
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => Some("pptx"),
        _ => None,
    }
}

pub const PDF_MIME_TYPE: &str = "application/pdf";
pub const DOCX_MIME_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document";

pub fn lowercase_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
}
