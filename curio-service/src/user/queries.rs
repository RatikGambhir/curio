use axum::{Extension, Json, extract::Path};
use serde::Serialize;

use crate::CurrentUser;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationResponse {
    id: String,
}

pub async fn get_conversation(
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Json<ConversationResponse> {
    Json(ConversationResponse { id })
}
