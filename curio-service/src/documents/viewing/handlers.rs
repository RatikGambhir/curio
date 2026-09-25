use axum::{
    Extension, Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;

use super::service::StoredDocumentText;
use crate::{
    CurrentUser,
    documents::{
        Viewing, domain::DocumentError, index::domain::ChunkRecord, store::domain::DocumentSummary,
    },
};

#[derive(Debug, Serialize)]
pub struct DocumentsResponse {
    documents: Vec<DocumentSummary>,
}

#[derive(Debug, Serialize)]
pub struct ChunksResponse {
    chunks: Vec<ChunkRecord>,
}

pub async fn list_documents(
    State(viewing): State<Viewing>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<DocumentsResponse>, DocumentError> {
    let documents = viewing.list(current_user.id()).await?;
    Ok(Json(DocumentsResponse { documents }))
}

pub async fn delete_document(
    State(viewing): State<Viewing>,
    Extension(current_user): Extension<CurrentUser>,
    Path(file_id): Path<String>,
) -> Result<StatusCode, DocumentError> {
    viewing.delete(current_user.id(), &file_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn document_chunks(
    State(viewing): State<Viewing>,
    Extension(current_user): Extension<CurrentUser>,
    Path(file_id): Path<String>,
) -> Result<Json<ChunksResponse>, DocumentError> {
    let chunks = viewing.chunks(current_user.id(), &file_id).await?;
    Ok(Json(ChunksResponse { chunks }))
}

pub async fn document_text(
    State(viewing): State<Viewing>,
    Extension(current_user): Extension<CurrentUser>,
    Path(file_id): Path<String>,
) -> Result<Json<StoredDocumentText>, DocumentError> {
    let document = viewing.load(current_user.id(), &file_id).await?;
    viewing.render_text(document).await.map(Json)
}

pub async fn document_pdf(
    State(viewing): State<Viewing>,
    Extension(current_user): Extension<CurrentUser>,
    Path(file_id): Path<String>,
) -> Result<Response, DocumentError> {
    let document = viewing.load(current_user.id(), &file_id).await?;
    let pdf_bytes = viewing.render_pdf(document).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("inline"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    Ok((headers, pdf_bytes).into_response())
}
