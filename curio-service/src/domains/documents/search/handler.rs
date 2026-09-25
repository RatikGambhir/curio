use std::sync::Arc;

use axum::{Extension, Json, extract::State};

use super::model::{KeywordSearchRequest, VectorSearchRequest};
use crate::{
    domains::documents::{
        Search,
        index::model::{KeywordChunkHit, VectorChunkHit},
        model::DocumentError,
    },
    shared::auth::CurrentUser,
};

pub async fn vector_search(
    State(search): State<Arc<Search>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<VectorSearchRequest>,
) -> Result<Json<Vec<VectorChunkHit>>, DocumentError> {
    search.vector(current_user.id(), request).await.map(Json)
}

pub async fn keyword_search(
    State(search): State<Arc<Search>>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<KeywordSearchRequest>,
) -> Result<Json<Vec<KeywordChunkHit>>, DocumentError> {
    search.keyword(current_user.id(), request).await.map(Json)
}
