//! Owner-scoped tag vocabulary, validation and task assignments.
use super::super::{
    filter::CreatedPage,
    model::{Tag, TaskError},
};
use super::{
    TaskTransaction,
    columns::{TAG_COLUMNS, TagAssignmentColumn, TagColumn, TaskColumn},
    queries::{advance, created_page},
    rows::TagRow,
};
use crate::adapters::postgres::query::SqlColumn;
use crate::adapters::postgres::query::{
    Delete, Function, Insert, Select, SqlType, Update, all, call, col, int, val,
};

impl TaskTransaction {
    pub async fn check_tags(&mut self, tags: &[String]) -> Result<(), TaskError> {
        let found: i64 = Select::from("task_tags")
            .columns([call(Function::Count, [all()])])
            .where_(col(TagColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(TagColumn::Id).eq_any(val(tags)))
            .build()?
            .build_query_scalar()
            .fetch_one(&mut *self.tx)
            .await?;
        if found == tags.len() as i64 {
            Ok(())
        } else {
            Err(TaskError::NotFound)
        }
    }
    pub async fn replace_tags(&mut self, task: &str, tags: &[String]) -> Result<(), TaskError> {
        Delete::from("task_tag_assignments")
            .where_(col(TagAssignmentColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(TagAssignmentColumn::TaskId).eq(val(task)))
            .build()?
            .build()
            .execute(&mut *self.tx)
            .await?;
        Insert::from_select(
            "task_tag_assignments",
            [
                TagAssignmentColumn::OwnerId,
                TagAssignmentColumn::TaskId,
                TagAssignmentColumn::TagId,
            ],
            Select::expressions([
                val(&self.owner),
                val(task),
                call(Function::Unnest, [val(tags).cast(SqlType::TextArray)]),
            ]),
        )
        .build()?
        .build()
        .execute(&mut *self.tx)
        .await?;
        Ok(())
    }
    pub async fn tags(&mut self, page: &CreatedPage) -> Result<Vec<Tag>, TaskError> {
        let query = created_page(
            Select::from("task_tags")
                .columns(TAG_COLUMNS)
                .where_(col(TagColumn::OwnerId).eq(val(&self.owner))),
            page,
            || col(TagColumn::CreatedAt),
            || col(TagColumn::Id),
        )?;
        let rows = query
            .build()?
            .build_query_as::<TagRow>()
            .fetch_all(&mut *self.tx)
            .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
    pub async fn create_tag(&mut self, name: &str, key: &str) -> Result<Tag, TaskError> {
        Ok(Insert::into("task_tags")
            .value(TagColumn::Id.value(uuid::Uuid::new_v4().to_string()))
            .value(TagColumn::OwnerId.value(&self.owner))
            .value(TagColumn::Name.value(name))
            .value(TagColumn::NameKey.value(key))
            .returning(TAG_COLUMNS)
            .build()?
            .build_query_as::<TagRow>()
            .fetch_one(&mut *self.tx)
            .await?
            .into())
    }
    pub async fn rename_tag(&mut self, id: &str, name: &str, key: &str) -> Result<Tag, TaskError> {
        Ok(Update::table("task_tags")
            .set(TagColumn::Name.value(name))
            .set(TagColumn::NameKey.value(key))
            .where_(col(TagColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(TagColumn::Id).eq(val(id)))
            .returning(TAG_COLUMNS)
            .build()?
            .build_query_as::<TagRow>()
            .fetch_optional(&mut *self.tx)
            .await?
            .ok_or(TaskError::NotFound)?
            .into())
    }
    pub async fn delete_tag(&mut self, id: &str) -> Result<(), TaskError> {
        // Removing assignments changes the aggregate protected by expectedVersion.
        let assigned = Select::from("task_tag_assignments")
            .columns([TagAssignmentColumn::TaskId])
            .where_(col(TagAssignmentColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(TagAssignmentColumn::TagId).eq(val(id)));
        Update::table("tasks")
            .set(TaskColumn::Version.expression(col(TaskColumn::Version).plus(int(1))))
            .set(TaskColumn::UpdatedAt.expression(advance(col(TaskColumn::UpdatedAt))))
            .where_(col(TaskColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(TaskColumn::Id).in_query(assigned))
            .build()?
            .build()
            .execute(&mut *self.tx)
            .await?;
        Delete::from("task_tags")
            .where_(col(TagColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(TagColumn::Id).eq(val(id)))
            .returning([TagColumn::Id])
            .build()?
            .build_query_scalar::<String>()
            .fetch_optional(&mut *self.tx)
            .await?
            .ok_or(TaskError::NotFound)
            .map(|_| ())
    }
}
