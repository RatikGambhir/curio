//! Task link reads, target writes and normalized link ordering.
use super::super::model::{Link, LinkInput, TaskError};
use super::{
    TaskTransaction,
    columns::{DerivedColumn, LINK_COLUMNS, LinkColumn, TaskColumn},
    queries::{advance, owned_task},
    rows::LinkRow,
};
use crate::adapters::postgres::query::{
    Delete, Function, Insert, Select, SqlColumn, SqlType, Update, call, col, int, rows_from, table,
    val,
};

impl TaskTransaction {
    pub async fn link(&mut self, task: &str, link: &str) -> Result<Link, TaskError> {
        Ok(Select::from(table("task_links").alias("l"))
            .columns(LINK_COLUMNS.map(|column| column.of("l")))
            .join(
                table("tasks").alias("t"),
                TaskColumn::OwnerId
                    .of("t")
                    .eq(LinkColumn::OwnerId.of("l"))
                    .and(TaskColumn::Id.of("t").eq(LinkColumn::TaskId.of("l"))),
            )
            .where_(LinkColumn::OwnerId.of("l").eq(val(&self.owner)))
            .where_(LinkColumn::TaskId.of("l").eq(val(task)))
            .where_(LinkColumn::Id.of("l").eq(val(link)))
            .build()?
            .build_query_as::<LinkRow>()
            .fetch_optional(&mut *self.tx)
            .await?
            .ok_or(TaskError::NotFound)?
            .into())
    }
    pub async fn create_link(&mut self, task: &str, input: &LinkInput) -> Result<Link, TaskError> {
        let source = Select::from("tasks")
            .columns([
                val(uuid::Uuid::new_v4().to_string()),
                col(TaskColumn::OwnerId),
                col(TaskColumn::Id),
                val(&input.kind),
                val(&input.label),
                val(&input.url),
                val(&input.note_id),
                int(0),
            ])
            .where_(col(TaskColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(TaskColumn::Id).eq(val(task)));
        Ok(Insert::from_select(
            "task_links",
            [
                LinkColumn::Id,
                LinkColumn::OwnerId,
                LinkColumn::TaskId,
                LinkColumn::Kind,
                LinkColumn::Label,
                LinkColumn::Url,
                LinkColumn::NoteId,
                LinkColumn::SortOrder,
            ],
            source,
        )
        .returning(LINK_COLUMNS)
        .build()?
        .build_query_as::<LinkRow>()
        .fetch_optional(&mut *self.tx)
        .await?
        .ok_or(TaskError::NotFound)?
        .into())
    }
    pub async fn edit_link(
        &mut self,
        task: &str,
        link: &str,
        input: &LinkInput,
    ) -> Result<(), TaskError> {
        Update::table("task_links")
            .set(LinkColumn::Kind.value(&input.kind))
            .set(LinkColumn::Label.value(&input.label))
            .set(LinkColumn::Url.value(&input.url))
            .set(LinkColumn::NoteId.value(&input.note_id))
            .set(LinkColumn::UpdatedAt.expression(advance(col(LinkColumn::UpdatedAt))))
            .where_(col(LinkColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(LinkColumn::TaskId).eq(val(task)))
            .where_(col(LinkColumn::Id).eq(val(link)))
            .returning([LinkColumn::Id])
            .build()?
            .build_query_scalar::<String>()
            .fetch_optional(&mut *self.tx)
            .await?
            .ok_or(TaskError::NotFound)
            .map(|_| ())
    }
    pub async fn delete_link(&mut self, task: &str, link: &str) -> Result<(), TaskError> {
        Delete::from(table("task_links").alias("l"))
            .where_(LinkColumn::OwnerId.of("l").eq(val(&self.owner)))
            .where_(LinkColumn::TaskId.of("l").eq(val(task)))
            .where_(LinkColumn::Id.of("l").eq(val(link)))
            .where_(owned_task(&self.owner, task).exists())
            .returning([LinkColumn::Id.of("l")])
            .build()?
            .build_query_scalar::<String>()
            .fetch_optional(&mut *self.tx)
            .await?
            .ok_or(TaskError::NotFound)
            .map(|_| ())
    }
    pub async fn link_ids(&mut self, task: &str) -> Result<Vec<String>, TaskError> {
        Ok(Select::from("task_links")
            .columns([LinkColumn::Id])
            .where_(col(LinkColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(LinkColumn::TaskId).eq(val(task)))
            .order_by(col(LinkColumn::SortOrder).asc())
            .order_by(col(LinkColumn::Id).asc())
            .build()?
            .build_query_scalar()
            .fetch_all(&mut *self.tx)
            .await?)
    }
    pub async fn link_ranks(&mut self, task: &str, ids: &[String]) -> Result<(), TaskError> {
        let ranks = rows_from(call(Function::Unnest, [val(ids).cast(SqlType::TextArray)]))
            .with_ordinality()
            .alias("r")
            .columns([DerivedColumn::Id, DerivedColumn::Rank]);
        Update::table(table("task_links").alias("l"))
            .set(LinkColumn::SortOrder.expression(DerivedColumn::Rank.of("r").minus(int(1))))
            .set(LinkColumn::UpdatedAt.expression(advance(LinkColumn::UpdatedAt.of("l"))))
            .from(ranks)
            .where_(LinkColumn::OwnerId.of("l").eq(val(&self.owner)))
            .where_(LinkColumn::TaskId.of("l").eq(val(task)))
            .where_(LinkColumn::Id.of("l").eq(DerivedColumn::Id.of("r")))
            .where_(
                LinkColumn::SortOrder
                    .of("l")
                    .ne(DerivedColumn::Rank.of("r").minus(int(1))),
            )
            .build()?
            .build()
            .execute(&mut *self.tx)
            .await?;
        Ok(())
    }
}
