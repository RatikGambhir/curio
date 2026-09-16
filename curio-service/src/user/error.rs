use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserError {
    Invalid(&'static str),
    Conflict(&'static str),
    Internal,
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            Self::Invalid(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "The user could not be saved.",
            ),
        };

        (status, Json(ErrorBody { error })).into_response()
    }
}
