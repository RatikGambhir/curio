use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// The one error the calendar's layers speak.
///
/// The service raises these; the `IntoResponse` impl is the single place they
/// become HTTP. Doing it once means handlers use `?` instead of building a
/// `(StatusCode, Json<...>)` tuple at each call site, and the body stays
/// `{"error": "…"}` for every failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalendarError {
    Invalid(&'static str),
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
            CalendarError::Invalid(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            CalendarError::Conflict(message) => (StatusCode::CONFLICT, message),
            CalendarError::UnknownUser => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "No profile exists for that user yet.",
            ),
            CalendarError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "The calendar is unavailable.",
            ),
        };

        (status, Json(ErrorBody { error })).into_response()
    }
}
