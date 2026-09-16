use chrono::{DateTime, Utc};

use crate::{
    diagnostics,
    task::{
        error::TaskError,
        model::{
            CreateTaskInput, DEFAULT_TASK_STATUS, ListTasksQuery, NewTask, TASK_PRIORITIES,
            TASK_STATUSES, TaskRecord,
        },
        repository::TaskRepository,
    },
};

#[derive(Clone)]
pub struct TaskService {
    repository: TaskRepository,
}

impl TaskService {
    pub fn new(repository: TaskRepository) -> Self {
        Self { repository }
    }

    pub async fn create_task(&self, input: CreateTaskInput) -> Result<TaskRecord, TaskError> {
        let user_id = required(
            &input.user_id,
            "A task needs the id of the user who owns it.",
        )?;
        let name = required(&input.name, "A task needs a name.")?;
        let id = optional(input.id.as_deref())
            .map(str::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let description = optional(input.description.as_deref());
        let status = optional(input.status.as_deref()).unwrap_or(DEFAULT_TASK_STATUS);
        let priority = optional(input.priority.as_deref());

        if !TASK_STATUSES.contains(&status) {
            return Err(TaskError::Invalid("That is not a known task status."));
        }
        if priority.is_some_and(|value| !TASK_PRIORITIES.contains(&value)) {
            return Err(TaskError::Invalid("That is not a known task priority."));
        }

        let set_at = match optional(input.set_at.as_deref()) {
            Some(value) => parse_timestamp(value, "The task start time could not be parsed.")?,
            None => Utc::now(),
        };
        let due_at = optional(input.due_at.as_deref())
            .map(|value| parse_timestamp(value, "The task due time could not be parsed."))
            .transpose()?;
        if due_at.is_some_and(|due_at| due_at < set_at) {
            return Err(TaskError::Invalid(
                "The task due time cannot fall before its start time.",
            ));
        }

        self.repository
            .insert(NewTask {
                id: &id,
                user_id,
                name,
                description,
                status,
                priority,
                active: input.active.unwrap_or(true),
                set_at: &set_at,
                due_at: due_at.as_ref(),
            })
            .await
            .map_err(|error| storage_error("task_create", error))
    }

    pub async fn list_tasks(&self, query: ListTasksQuery) -> Result<Vec<TaskRecord>, TaskError> {
        let user_id = required(
            &query.user_id,
            "A task list needs the id of the user who owns it.",
        )?;

        self.repository
            .list_by_user(user_id)
            .await
            .map_err(|error| storage_error("task_list", error))
    }
}

fn parse_timestamp(value: &str, message: &'static str) -> Result<DateTime<Utc>, TaskError> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| TaskError::Invalid(message))
}

fn storage_error(operation: &'static str, error: sqlx::Error) -> TaskError {
    if let Some(database_error) = error.as_database_error() {
        if database_error.is_unique_violation() {
            return TaskError::Conflict("A task with that id already exists.");
        }
        if database_error.is_foreign_key_violation() {
            return TaskError::UnknownUser;
        }
    }

    diagnostics::log_database_error(operation, &error);
    TaskError::Internal
}

fn required<'a>(value: &'a str, message: &'static str) -> Result<&'a str, TaskError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(TaskError::Invalid(message));
    }
    Ok(value)
}

fn optional(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}
