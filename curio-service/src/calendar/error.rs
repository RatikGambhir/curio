use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// The calendar module does enough validation that returning
/// `(StatusCode, Json<...>)` tuples from every call site gets noisy, so the
/// mapping to HTTP lives here once and handlers use `?`.
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

impl CalendarError {
    /// Collapses a storage failure into a client-facing error. The unique and
    /// foreign key violations are the two the caller can actually act on;
    /// everything else is logged here and sanitized to `Internal`, because the
    /// cause belongs to operators and the message belongs to the client.
    pub fn from_storage(context: &str, error: sqlx::Error) -> Self {
        if let sqlx::Error::Database(ref database_error) = error {
            if database_error.is_unique_violation() {
                return CalendarError::Conflict("An event with that id already exists.");
            }
            if database_error.is_foreign_key_violation() {
                return CalendarError::UnknownUser;
            }
        }

        // The service has no logging crate yet and main.rs uses println!, so
        // this matches rather than dropping the cause silently.
        eprintln!("curio-service: {context} failed: {error}");
        CalendarError::Internal
    }
}
