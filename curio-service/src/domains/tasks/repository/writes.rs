//! Core task create, optimistic update and subtree delete operations.
use super::super::model::{Task, TaskError};
use super::{TaskTransaction, columns::TaskColumn, queries::advance};
use crate::adapters::postgres::query::{
    Delete, Insert, Select, SqlColumn, Update, col, int, table, val,
};

impl TaskTransaction {
    pub async fn insert(&mut self, task: &Task) -> Result<(), TaskError> {
        Insert::into("tasks")
            .value(TaskColumn::Id.value(&task.id))
            .value(TaskColumn::OwnerId.value(&self.owner))
            .value(TaskColumn::SpaceId.value(&task.space_id))
            .value(TaskColumn::ParentId.value(&task.parent_id))
            .value(TaskColumn::Title.value(&task.title))
            .value(TaskColumn::Description.value(&task.description))
            .value(TaskColumn::Status.value(&task.status))
            .value(TaskColumn::Priority.value(&task.priority))
            .value(TaskColumn::Flagged.value(task.flagged))
            .value(TaskColumn::DueOn.value(task.due_on))
            .value(TaskColumn::DueAt.value(task.due_at))
            .value(TaskColumn::DueTimezone.value(&task.due_timezone))
            .value(TaskColumn::CompletedAt.value(task.completed_at))
            .value(TaskColumn::SortOrder.value(task.sort_order))
            .returning([TaskColumn::Id])
            .build()?
            .build()
            .fetch_one(&mut *self.tx)
            .await?;
        Ok(())
    }
    pub async fn update(&mut self, task: &Task, expected: i64) -> Result<(), TaskError> {
        let found = Update::table("tasks")
            .set(TaskColumn::SpaceId.value(&task.space_id))
            .set(TaskColumn::ParentId.value(&task.parent_id))
            .set(TaskColumn::Title.value(&task.title))
            .set(TaskColumn::Description.value(&task.description))
            .set(TaskColumn::Status.value(&task.status))
            .set(TaskColumn::Priority.value(&task.priority))
            .set(TaskColumn::Flagged.value(task.flagged))
            .set(TaskColumn::DueOn.value(task.due_on))
            .set(TaskColumn::DueAt.value(task.due_at))
            .set(TaskColumn::DueTimezone.value(&task.due_timezone))
            .set(TaskColumn::CompletedAt.value(task.completed_at))
            .set(TaskColumn::Version.expression(col(TaskColumn::Version).plus(int(1))))
            .set(TaskColumn::UpdatedAt.expression(advance(col(TaskColumn::UpdatedAt))))
            .where_(col(TaskColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(TaskColumn::Id).eq(val(&task.id)))
            .where_(col(TaskColumn::Version).eq(val(expected)))
            .returning([TaskColumn::Id])
            .build()?
            .build_query_scalar::<String>()
            .fetch_optional(&mut *self.tx)
            .await?;
        found.ok_or(TaskError::Conflict).map(|_| ())
    }
    pub async fn delete(
        &mut self,
        task: &str,
        expected: i64,
        descendants: &[String],
    ) -> Result<(), TaskError> {
        // The root version stays in the same write predicate as subtree deletion.
        let root = Select::from(table("tasks").alias("root"))
            .columns([int(1)])
            .where_(TaskColumn::OwnerId.of("root").eq(val(&self.owner)))
            .where_(TaskColumn::Id.of("root").eq(val(task)))
            .where_(TaskColumn::Version.of("root").eq(val(expected)));
        let deleted: Vec<String> = Delete::from("tasks")
            .where_(col(TaskColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(TaskColumn::Id).eq_any(val(descendants)))
            .where_(root.exists())
            .returning([TaskColumn::Id])
            .build()?
            .build_query_scalar()
            .fetch_all(&mut *self.tx)
            .await?;
        if deleted.is_empty() {
            Err(TaskError::Conflict)
        } else {
            Ok(())
        }
    }
}
