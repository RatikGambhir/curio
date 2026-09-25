//! HTTP mapping shared by every documents subdomain.
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use super::domain::{DocumentError, INTERNAL_MESSAGE, UNKNOWN_OWNER_MESSAGE};

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for DocumentError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::Invalid(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            Self::PayloadTooLarge(message) => (StatusCode::PAYLOAD_TOO_LARGE, message),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, message),
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
            Self::UnknownOwner => (
                StatusCode::UNPROCESSABLE_ENTITY,
                UNKNOWN_OWNER_MESSAGE.to_owned(),
            ),
            Self::Unavailable(message) => (StatusCode::SERVICE_UNAVAILABLE, message),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                INTERNAL_MESSAGE.to_owned(),
            ),
        };

        (status, Json(ErrorBody { error })).into_response()
    }
}
