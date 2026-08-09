use axum::{Extension, Json, extract::Path, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::CurrentUser;

#[derive(Deserialize)]
pub struct ConversationRequest {
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationResponse {
    id: String,
    message: String,
}

pub async fn create_conversation(
    Extension(_current_user): Extension<CurrentUser>,
    Json(request): Json<ConversationRequest>,
) -> (StatusCode, Json<ConversationResponse>) {
    (
        StatusCode::CREATED,
        Json(ConversationResponse {
            id: "placeholder-conversation".to_owned(),
            message: request.message,
        }),
    )
}

pub async fn update_conversation(
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
    Json(request): Json<ConversationRequest>,
) -> Json<ConversationResponse> {
    Json(ConversationResponse {
        id,
        message: request.message,
    })
}
