use crate::{
    postgres_test_support::PostgresFixture,
    task::{
        error::TaskError,
        model::{CreateTaskInput, ListTasksQuery},
        repository::TaskRepository,
        service::TaskService,
    },
};

async fn service(test_name: &str) -> Option<(TaskService, PostgresFixture)> {
    let postgres = PostgresFixture::provision(test_name).await?;
    let service = TaskService::new(TaskRepository::new(postgres.database().clone()));
    Some((service, postgres))
}

fn create_input(name: &str) -> CreateTaskInput {
    CreateTaskInput {
        id: None,
        user_id: "user-a".to_owned(),
        name: name.to_owned(),
        description: None,
        status: None,
        priority: None,
        active: None,
        set_at: Some("2026-09-01T12:00:00-05:00".to_owned()),
        due_at: Some("2026-09-01T18:00:00Z".to_owned()),
    }
}

#[tokio::test]
async fn validates_required_enums_and_timestamps_before_storage() {
    let Some((service, postgres)) = service("task_service_validation").await else {
        return;
    };

    assert_eq!(
        service.create_task(create_input(" ")).await,
        Err(TaskError::Invalid("A task needs a name."))
    );

    let mut input = create_input("Task");
    input.status = Some("unknown".to_owned());
    assert_eq!(
        service.create_task(input).await,
        Err(TaskError::Invalid("That is not a known task status."))
    );

    let mut input = create_input("Task");
    input.priority = Some("urgent".to_owned());
    assert_eq!(
        service.create_task(input).await,
        Err(TaskError::Invalid("That is not a known task priority."))
    );

    let mut input = create_input("Task");
    input.set_at = Some("tomorrow".to_owned());
    assert_eq!(
        service.create_task(input).await,
        Err(TaskError::Invalid(
            "The task start time could not be parsed."
        ))
    );

    let mut input = create_input("Task");
    input.due_at = Some("2026-09-01T16:59:59Z".to_owned());
    assert_eq!(
        service.create_task(input).await,
        Err(TaskError::Invalid(
            "The task due time cannot fall before its start time."
        ))
    );

    postgres.cleanup().await;
}

#[tokio::test]
async fn applies_defaults_normalization_and_storage_error_mapping() {
    let Some((service, postgres)) = service("task_service_defaults_and_errors").await else {
        return;
    };

    assert_eq!(
        service.create_task(create_input("Task")).await,
        Err(TaskError::UnknownUser)
    );
    seed_user(&postgres, "user-a").await;

    let mut input = create_input("  Draft brief  ");
    input.id = Some("  task-1  ".to_owned());
    input.description = Some("   ".to_owned());
    input.status = Some("   ".to_owned());
    input.priority = Some("  high  ".to_owned());
    input.active = None;
    let created = service.create_task(input).await.unwrap();

    assert_eq!(created.id, "task-1");
    assert_eq!(created.name, "Draft brief");
    assert_eq!(created.description, None);
    assert_eq!(created.status, "scheduled");
    assert_eq!(created.priority.as_deref(), Some("high"));
    assert!(created.active);
    assert_eq!(created.set_at.to_rfc3339(), "2026-09-01T17:00:00+00:00");

    let tasks = service
        .list_tasks(ListTasksQuery {
            user_id: " user-a ".to_owned(),
        })
        .await
        .unwrap();
    assert_eq!(tasks.len(), 1);

    let mut duplicate = create_input("Duplicate");
    duplicate.id = Some("task-1".to_owned());
    assert_eq!(
        service.create_task(duplicate).await,
        Err(TaskError::Conflict("A task with that id already exists."))
    );

    postgres.cleanup().await;
}

async fn seed_user(postgres: &PostgresFixture, id: &str) {
    sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(id)
        .bind(format!("{id}@example.com"))
        .execute(postgres.database().pool())
        .await
        .unwrap();
}
