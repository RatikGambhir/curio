use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    CurrentUser,
    user::{
        error::UserError,
        model::{
            ConversationLookupResponse, ConversationRequest, ConversationResponse, SaveUserRequest,
            UserRecord,
        },
        service::UserService,
    },
};

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

pub async fn get_conversation(
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Json<ConversationLookupResponse> {
    Json(ConversationLookupResponse { id })
}

pub async fn save_user(
    State(service): State<UserService>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(request): Json<SaveUserRequest>,
) -> Result<Json<UserRecord>, UserError> {
    service.save_user(request).await.map(Json)
}
