use chrono::{TimeZone, Utc};

use crate::{
    postgres_test_support::PostgresFixture,
    task::{model::NewTask, repository::TaskRepository},
};

#[tokio::test]
async fn inserts_native_values_and_lists_in_stable_order() {
    let Some(postgres) = PostgresFixture::provision("task_repository_insert_and_list").await else {
        return;
    };
    seed_user(&postgres, "user-a").await;
    let repository = TaskRepository::new(postgres.database().clone());
    let set_at = Utc.with_ymd_and_hms(2026, 9, 1, 10, 0, 0).single().unwrap();
    let due_at = Utc.with_ymd_and_hms(2026, 9, 2, 10, 0, 0).single().unwrap();
    let hostile = "literal'); DELETE FROM tasks; --";

    for id in ["task-b", "task-a"] {
        repository
            .insert(NewTask {
                id,
                user_id: "user-a",
                name: hostile,
                description: Some(hostile),
                status: "scheduled",
                priority: Some("high"),
                active: false,
                set_at: &set_at,
                due_at: Some(&due_at),
            })
            .await
            .unwrap();
    }
    sqlx::query("UPDATE tasks SET created_at = '2026-09-01T12:00:00Z' WHERE user_id = $1")
        .bind("user-a")
        .execute(postgres.database().pool())
        .await
        .unwrap();

    let tasks = repository.list_by_user("user-a").await.unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].id, "task-a");
    assert_eq!(tasks[1].id, "task-b");
    assert_eq!(tasks[0].name, hostile);
    assert_eq!(tasks[0].description.as_deref(), Some(hostile));
    assert_eq!(tasks[0].priority.as_deref(), Some("high"));
    assert!(!tasks[0].active);
    assert_eq!(tasks[0].set_at, set_at);
    assert_eq!(tasks[0].due_at, Some(due_at));

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind("user-a")
        .execute(postgres.database().pool())
        .await
        .unwrap();
    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks")
        .fetch_one(postgres.database().pool())
        .await
        .unwrap();
    assert_eq!(remaining, 0);

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
