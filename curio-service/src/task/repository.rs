use crate::{
    core::sql::Sql,
    database::Database,
    task::model::{NewTask, TaskRecord},
};

#[derive(Clone)]
pub struct TaskRepository {
    database: Database,
}

impl TaskRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn insert(&self, task: NewTask<'_>) -> Result<TaskRecord, sqlx::Error> {
        Sql::insert_into("tasks")
            .set("id", task.id)
            .set("user_id", task.user_id)
            .set("name", task.name)
            .set("description", task.description)
            .set("status", task.status)
            .set("priority", task.priority)
            .set("active", task.active)
            .set("set_at", task.set_at)
            .set("due_at", task.due_at)
            .returning(TaskRecord::COLUMNS)
            .fetch_one(self.database.pool())
            .await
    }

    pub async fn list_by_user(&self, user_id: &str) -> Result<Vec<TaskRecord>, sqlx::Error> {
        Sql::select(TaskRecord::COLUMNS)
            .from("tasks")
            .filter("user_id = ?", user_id)
            .order_by("created_at DESC, id ASC")
            .fetch_all(self.database.pool())
            .await
    }
}
