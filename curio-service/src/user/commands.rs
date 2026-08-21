use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{
    CurrentUser,
    database::{Database, UserRecord},
};

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveUserRequest {
    id: String,
    name: String,
    email: String,
    avatar_url: Option<String>,
}

#[derive(Serialize)]
pub struct UserErrorResponse {
    error: &'static str,
}

type UserError = (StatusCode, Json<UserErrorResponse>);

fn user_error(status: StatusCode, error: &'static str) -> UserError {
    (status, Json(UserErrorResponse { error }))
}

pub async fn save_user(
    State(database): State<Database>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(request): Json<SaveUserRequest>,
) -> Result<Json<UserRecord>, UserError> {
    let id = request.id.trim();
    let name = request.name.trim();
    let email = request.email.trim();
    let avatar_url = request
        .avatar_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if id.is_empty() || name.is_empty() || email.is_empty() {
        return Err(user_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "A user id, name, and email are required.",
        ));
    }

    match database.save_user(id, name, email, avatar_url).await {
        Ok(user) => Ok(Json(user)),
        Err(sqlx::Error::Database(error)) if error.is_unique_violation() => Err(user_error(
            StatusCode::CONFLICT,
            "That email is already in use by another account.",
        )),
        Err(_) => Err(user_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The user could not be saved.",
        )),
    }
}
