use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{
        Path, Query, State,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use super::{
    Service,
    model::{CreateSpaceInput, ListSpacesQuery, SpaceError},
};
use crate::shared::auth::CurrentUser;

pub async fn create(
    State(service): State<Arc<Service>>,
    Extension(user): Extension<CurrentUser>,
    input: Result<Json<CreateSpaceInput>, JsonRejection>,
) -> Response {
    let input = match input {
        Ok(Json(input)) => input,
        Err(error) => {
            return error_response(
                error.status(),
                match error.status() {
                    StatusCode::UNPROCESSABLE_ENTITY => "Invalid or unknown Space fields.",
                    StatusCode::PAYLOAD_TOO_LARGE => "Space request body is too large.",
                    StatusCode::UNSUPPORTED_MEDIA_TYPE => "Content-Type must be application/json.",
                    _ => "Invalid JSON body.",
                },
            );
        }
    };
    match service.create(user.id(), input).await {
        Ok(space) => (StatusCode::CREATED, Json(space)).into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn list(
    State(service): State<Arc<Service>>,
    Extension(user): Extension<CurrentUser>,
    query: Result<Query<ListSpacesQuery>, QueryRejection>,
) -> Response {
    let Ok(Query(query)) = query else {
        return SpaceError::InvalidQuery.into_response();
    };
    match service.list(user.id(), query).await {
        Ok(page) => Json(page).into_response(),
        Err(error) => error.into_response(),
    }
}

pub async fn delete(
    State(service): State<Arc<Service>>,
    Extension(user): Extension<CurrentUser>,
    path: Result<Path<String>, PathRejection>,
) -> Response {
    let Ok(Path(id)) = path else {
        return error_response(StatusCode::BAD_REQUEST, "Invalid Space path.");
    };
    match service.delete(user.id(), &id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => error.into_response(),
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}

fn error_response(status: StatusCode, error: &'static str) -> Response {
    (status, Json(ErrorBody { error })).into_response()
}

impl IntoResponse for SpaceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Invalid(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            Self::InvalidQuery => (StatusCode::BAD_REQUEST, "Invalid Spaces query or cursor."),
            Self::DuplicateName => (
                StatusCode::CONFLICT,
                "A Space with that name already exists.",
            ),
            Self::UnknownUser => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "No profile exists for that user yet.",
            ),
            Self::NotFound => (StatusCode::NOT_FOUND, "Space not found."),
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Spaces are unavailable."),
        };
        error_response(status, message)
    }
}
