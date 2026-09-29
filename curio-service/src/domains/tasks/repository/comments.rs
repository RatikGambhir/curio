//! Owner-authored task comments and their paginated reads.
use super::super::{
    filter::CreatedPage,
    model::{Comment, TaskError},
};
use super::{
    TaskTransaction,
    columns::{COMMENT_COLUMNS, CommentColumn, TaskColumn},
    queries::{created_page, owned_task},
    rows::CommentRow,
};
use crate::adapters::postgres::query::{
    Delete, Function, Insert, Select, SqlColumn, SqlType, Update, call, col, millisecond, table,
    val,
};

impl TaskTransaction {
    pub async fn comments(
        &mut self,
        task: &str,
        page: &CreatedPage,
    ) -> Result<Vec<Comment>, TaskError> {
        let query = Select::from(table("task_comments").alias("c"))
            .columns(COMMENT_COLUMNS.map(|column| column.of("c")))
            .join(
                table("tasks").alias("t"),
                TaskColumn::OwnerId
                    .of("t")
                    .eq(CommentColumn::OwnerId.of("c"))
                    .and(TaskColumn::Id.of("t").eq(CommentColumn::TaskId.of("c"))),
            )
            .where_(CommentColumn::OwnerId.of("c").eq(val(&self.owner)))
            .where_(CommentColumn::TaskId.of("c").eq(val(task)));
        let rows = created_page(
            query,
            page,
            || CommentColumn::CreatedAt.of("c"),
            || CommentColumn::Id.of("c"),
        )?
        .build()?
        .build_query_as::<CommentRow>()
        .fetch_all(&mut *self.tx)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
    pub async fn create_comment(&mut self, task: &str, body: &str) -> Result<Comment, TaskError> {
        let source = Select::from("tasks")
            .columns([
                val(uuid::Uuid::new_v4().to_string()),
                col(TaskColumn::OwnerId),
                col(TaskColumn::Id),
                col(TaskColumn::OwnerId),
                val(body),
            ])
            .where_(col(TaskColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(TaskColumn::Id).eq(val(task)));
        Ok(Insert::from_select(
            "task_comments",
            [
                CommentColumn::Id,
                CommentColumn::OwnerId,
                CommentColumn::TaskId,
                CommentColumn::AuthorId,
                CommentColumn::Body,
            ],
            source,
        )
        .returning(COMMENT_COLUMNS)
        .build()?
        .build_query_as::<CommentRow>()
        .fetch_optional(&mut *self.tx)
        .await?
        .ok_or(TaskError::NotFound)?
        .into())
    }
    pub async fn edit_comment(
        &mut self,
        task: &str,
        comment: &str,
        body: &str,
    ) -> Result<Comment, TaskError> {
        let edited_at = call(
            Function::Greatest,
            [
                call(Function::ClockTimestamp, []).cast(SqlType::TimestampMillis),
                call(
                    Function::Coalesce,
                    [col(CommentColumn::EditedAt), col(CommentColumn::CreatedAt)],
                )
                .plus(millisecond()),
            ],
        );
        Ok(Update::table(table("task_comments").alias("c"))
            .set(CommentColumn::Body.value(body))
            .set(CommentColumn::EditedAt.expression(edited_at))
            .where_(CommentColumn::OwnerId.of("c").eq(val(&self.owner)))
            .where_(CommentColumn::TaskId.of("c").eq(val(task)))
            .where_(CommentColumn::Id.of("c").eq(val(comment)))
            .where_(CommentColumn::AuthorId.of("c").eq(val(&self.owner)))
            .where_(owned_task(&self.owner, task).exists())
            .returning(COMMENT_COLUMNS.map(|column| column.of("c")))
            .build()?
            .build_query_as::<CommentRow>()
            .fetch_optional(&mut *self.tx)
            .await?
            .ok_or(TaskError::NotFound)?
            .into())
    }
    pub async fn delete_comment(&mut self, task: &str, comment: &str) -> Result<(), TaskError> {
        Delete::from(table("task_comments").alias("c"))
            .where_(CommentColumn::OwnerId.of("c").eq(val(&self.owner)))
            .where_(CommentColumn::TaskId.of("c").eq(val(task)))
            .where_(CommentColumn::Id.of("c").eq(val(comment)))
            .where_(CommentColumn::AuthorId.of("c").eq(val(&self.owner)))
            .where_(owned_task(&self.owner, task).exists())
            .returning([CommentColumn::Id.of("c")])
            .build()?
            .build_query_scalar::<String>()
            .fetch_optional(&mut *self.tx)
            .await?
            .ok_or(TaskError::NotFound)
            .map(|_| ())
    }
}
