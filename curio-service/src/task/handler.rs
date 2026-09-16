use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::Serialize;

use crate::{
    CurrentUser,
    task::{
        error::TaskError,
        model::{CreateTaskInput, ListTasksQuery, TaskRecord},
        service::TaskService,
    },
};

#[derive(Debug, Serialize)]
pub struct TasksResponse {
    tasks: Vec<TaskRecord>,
}

pub async fn create_task(
    State(service): State<TaskService>,
    Extension(current_user): Extension<CurrentUser>,
    Json(input): Json<CreateTaskInput>,
) -> Result<(StatusCode, Json<TaskRecord>), TaskError> {
    authorize_owner(&current_user, &input.user_id)?;
    let task = service.create_task(input).await?;
    Ok((StatusCode::CREATED, Json(task)))
}

pub async fn list_tasks(
    State(service): State<TaskService>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ListTasksQuery>,
) -> Result<Json<TasksResponse>, TaskError> {
    authorize_owner(&current_user, &query.user_id)?;
    let tasks = service.list_tasks(query).await?;
    Ok(Json(TasksResponse { tasks }))
}

fn authorize_owner(current_user: &CurrentUser, requested_user_id: &str) -> Result<(), TaskError> {
    if requested_user_id.trim().is_empty() || current_user.owns(requested_user_id) {
        Ok(())
    } else {
        Err(TaskError::Forbidden)
    }
}
