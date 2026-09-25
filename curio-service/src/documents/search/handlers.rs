use axum::{Extension, Json, extract::State};

use super::domain::{KeywordSearchRequest, VectorSearchRequest};
use crate::{
    CurrentUser,
    documents::{
        Search,
        domain::DocumentError,
        index::domain::{KeywordChunkHit, VectorChunkHit},
    },
};

pub async fn vector_search(
    State(search): State<Search>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<VectorSearchRequest>,
) -> Result<Json<Vec<VectorChunkHit>>, DocumentError> {
    search.vector(current_user.id(), request).await.map(Json)
}

pub async fn keyword_search(
    State(search): State<Search>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<KeywordSearchRequest>,
) -> Result<Json<Vec<KeywordChunkHit>>, DocumentError> {
    search.keyword(current_user.id(), request).await.map(Json)
}
