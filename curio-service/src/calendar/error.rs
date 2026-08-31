use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalendarError {
    Invalid(&'static str),
    Forbidden,
    Conflict(&'static str),
    UnknownUser,
    Internal,
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

impl IntoResponse for CalendarError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            Self::Invalid(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "Calendar events can only be accessed by their owner.",
            ),
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
            Self::UnknownUser => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "No profile exists for that user yet.",
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "The calendar is unavailable.",
            ),
        };

        (status, Json(ErrorBody { error })).into_response()
    }
}
