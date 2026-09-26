//! HTTP extraction and feature-local sanitized errors.
use super::{
    filter::{DeleteQuery, ListQuery, PageQuery},
    model::*,
    service::{CommentsResponse, TagsResponse, TaskService, TasksResponse},
};
use crate::shared::auth::CurrentUser;
use axum::{
    Extension, Json,
    extract::{
        Path, Query, State,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;

pub struct HttpError(StatusCode, &'static str);
#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
}
impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (self.0, Json(ErrorBody { error: self.1 })).into_response()
    }
}
impl From<TaskError> for HttpError {
    fn from(error: TaskError) -> Self {
        let (status, message) = match error {
            TaskError::Invalid(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            TaskError::Query => (StatusCode::BAD_REQUEST, "Invalid task query or cursor."),
            TaskError::NotFound => (StatusCode::NOT_FOUND, "Task resource not found."),
            TaskError::UnknownUser => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "No profile exists for that user yet.",
            ),
            TaskError::Conflict => (
                StatusCode::CONFLICT,
                "Task version has changed. Reload and retry.",
            ),
            TaskError::DuplicateTag => {
                (StatusCode::CONFLICT, "A tag with that name already exists.")
            }
            TaskError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Tasks are unavailable."),
        };
        Self(status, message)
    }
}
fn body<T>(value: Result<Json<T>, JsonRejection>) -> Result<T, HttpError> {
    value.map(|Json(value)| value).map_err(|error| {
        HttpError(
            error.status(),
            match error.status() {
                StatusCode::UNPROCESSABLE_ENTITY => "Invalid or unknown request fields.",
                StatusCode::PAYLOAD_TOO_LARGE => "Task request body is too large.",
                StatusCode::UNSUPPORTED_MEDIA_TYPE => "Content-Type must be application/json.",
                _ => "Invalid JSON body.",
            },
        )
    })
}
fn query<T: DeserializeOwned>(value: Result<Query<T>, QueryRejection>) -> Result<T, HttpError> {
    value
        .map(|Query(value)| value)
        .map_err(|_| TaskError::Query.into())
}
fn path<T>(value: Result<Path<T>, PathRejection>) -> Result<T, HttpError> {
    value
        .map(|Path(value)| value)
        .map_err(|_| HttpError(StatusCode::BAD_REQUEST, "Invalid task path."))
}

pub async fn create(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    input: Result<Json<CreateTask>, JsonRejection>,
) -> Result<(StatusCode, Json<Task>), HttpError> {
    Ok((
        StatusCode::CREATED,
        Json(service.create(user.id(), body(input)?).await?),
    ))
}

pub async fn list(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    input: Result<Query<ListQuery>, QueryRejection>,
) -> Result<Json<TasksResponse>, HttpError> {
    Ok(Json(service.list(user.id(), query(input)?).await?))
}

pub async fn get(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    target: Result<Path<String>, PathRejection>,
) -> Result<Json<Task>, HttpError> {
    let id = path(target)?;
    Ok(Json(service.get(user.id(), &id).await?))
}

pub async fn patch(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    target: Result<Path<String>, PathRejection>,
    input: Result<Json<PatchTask>, JsonRejection>,
) -> Result<Json<Task>, HttpError> {
    let id = path(target)?;
    Ok(Json(service.patch(user.id(), &id, body(input)?).await?))
}

pub async fn delete(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    target: Result<Path<String>, PathRejection>,
    input: Result<Query<DeleteQuery>, QueryRejection>,
) -> Result<StatusCode, HttpError> {
    let id = path(target)?;
    service
        .delete(user.id(), &id, query(input)?.expected_version)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn tags(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    input: Result<Query<PageQuery>, QueryRejection>,
) -> Result<Json<TagsResponse>, HttpError> {
    Ok(Json(service.tags(user.id(), query(input)?).await?))
}

pub async fn create_tag(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    input: Result<Json<TagInput>, JsonRejection>,
) -> Result<(StatusCode, Json<Tag>), HttpError> {
    Ok((
        StatusCode::CREATED,
        Json(service.create_tag(user.id(), body(input)?).await?),
    ))
}

pub async fn rename_tag(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    target: Result<Path<String>, PathRejection>,
    input: Result<Json<TagInput>, JsonRejection>,
) -> Result<Json<Tag>, HttpError> {
    let id = path(target)?;
    Ok(Json(
        service.rename_tag(user.id(), &id, body(input)?).await?,
    ))
}

pub async fn delete_tag(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    target: Result<Path<String>, PathRejection>,
) -> Result<StatusCode, HttpError> {
    let id = path(target)?;
    service.delete_tag(user.id(), &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn comments(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    target: Result<Path<String>, PathRejection>,
    input: Result<Query<PageQuery>, QueryRejection>,
) -> Result<Json<CommentsResponse>, HttpError> {
    let id = path(target)?;
    Ok(Json(service.comments(user.id(), &id, query(input)?).await?))
}

pub async fn create_comment(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    target: Result<Path<String>, PathRejection>,
    input: Result<Json<CommentInput>, JsonRejection>,
) -> Result<(StatusCode, Json<Comment>), HttpError> {
    let id = path(target)?;
    Ok((
        StatusCode::CREATED,
        Json(service.create_comment(user.id(), &id, body(input)?).await?),
    ))
}

pub async fn edit_comment(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    target: Result<Path<(String, String)>, PathRejection>,
    input: Result<Json<CommentInput>, JsonRejection>,
) -> Result<Json<Comment>, HttpError> {
    let (id, child) = path(target)?;
    Ok(Json(
        service
            .edit_comment(user.id(), &id, &child, body(input)?)
            .await?,
    ))
}

pub async fn delete_comment(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    target: Result<Path<(String, String)>, PathRejection>,
) -> Result<StatusCode, HttpError> {
    let (id, child) = path(target)?;
    service.delete_comment(user.id(), &id, &child).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_link(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    target: Result<Path<String>, PathRejection>,
    input: Result<Json<LinkInput>, JsonRejection>,
) -> Result<(StatusCode, Json<Link>), HttpError> {
    let id = path(target)?;
    Ok((
        StatusCode::CREATED,
        Json(service.create_link(user.id(), &id, body(input)?).await?),
    ))
}

pub async fn edit_link(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    target: Result<Path<(String, String)>, PathRejection>,
    input: Result<Json<PatchLink>, JsonRejection>,
) -> Result<Json<Link>, HttpError> {
    let (id, child) = path(target)?;
    Ok(Json(
        service
            .edit_link(user.id(), &id, &child, body(input)?)
            .await?,
    ))
}

pub async fn delete_link(
    State(service): State<Arc<TaskService>>,
    Extension(user): Extension<CurrentUser>,
    target: Result<Path<(String, String)>, PathRejection>,
) -> Result<StatusCode, HttpError> {
    let (id, child) = path(target)?;
    service.delete_link(user.id(), &id, &child).await?;
    Ok(StatusCode::NO_CONTENT)
}
