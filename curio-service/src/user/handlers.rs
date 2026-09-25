//! HTTP mapping for profile use cases.
use super::{
    Service,
    domain::{SaveUserRequest, UserError, UserRecord},
};
use crate::CurrentUser;
use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub async fn save_user(
    State(service): State<Service>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(input): Json<SaveUserRequest>,
) -> Result<Json<UserRecord>, UserError> {
    service.save_user(input).await.map(Json)
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            Self::Invalid => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "A user id, name, and email are required.",
            ),
            Self::EmailConflict => (
                StatusCode::CONFLICT,
                "That email is already in use by another account.",
            ),
            Self::Unavailable => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "The user could not be saved.",
            ),
        };
        (status, Json(ErrorBody { error })).into_response()
    }
}
