//! Tasks persistence entry point and transaction lifecycle.
mod columns;
mod comments;
mod hierarchy;
mod links;
mod ordering;
mod queries;
mod reads;
mod references;
mod rows;
mod tags;
mod writes;

pub(crate) use hierarchy::detach_space_tasks;

use super::model::TaskError;
use crate::{
    adapters::postgres::{
        client::Database,
        query::{Select, col, repeatable_read, val},
    },
    domains::users::repository::UserColumn,
};
use sqlx::{Postgres, Transaction};

pub struct TaskRepository {
    database: Database,
}
pub struct TaskTransaction {
    tx: Transaction<'static, Postgres>,
    owner: String,
}
impl TaskRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
    pub async fn write(&self, owner: &str, creating: bool) -> Result<TaskTransaction, TaskError> {
        let mut tx = self.database.pool().begin().await?;
        let found = Select::from("users")
            .columns([UserColumn::Id])
            .where_(col(UserColumn::Id).eq(val(owner)))
            .for_update()
            .build()?
            .build_query_scalar::<String>()
            .fetch_optional(&mut *tx)
            .await?;
        if found.is_none() {
            return Err(if creating {
                TaskError::UnknownUser
            } else {
                TaskError::NotFound
            });
        }
        Ok(TaskTransaction {
            tx,
            owner: owner.to_owned(),
        })
    }
    pub async fn read(&self, owner: &str) -> Result<TaskTransaction, TaskError> {
        let mut tx = self.database.pool().begin().await?;
        repeatable_read().build().execute(&mut *tx).await?;
        Ok(TaskTransaction {
            tx,
            owner: owner.to_owned(),
        })
    }
}
impl TaskTransaction {
    pub async fn commit(self) -> Result<(), TaskError> {
        self.tx.commit().await.map_err(Into::into)
    }
}

impl From<sqlx::Error> for TaskError {
    fn from(error: sqlx::Error) -> Self {
        if let sqlx::Error::Database(ref db) = error
            && db.is_unique_violation()
            && db.constraint() == Some("task_tags_owner_name_key")
        {
            return Self::DuplicateTag;
        }
        crate::shared::diagnostics::log_database_error("tasks_storage", &error);
        Self::Internal
    }
}
