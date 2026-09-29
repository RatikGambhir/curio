//! Task sibling selection and normalized, versioned manual ranks.
use super::super::model::TaskError;
use super::{
    TaskTransaction,
    columns::{DerivedColumn, TaskColumn},
    queries::advance,
};
use crate::adapters::postgres::query::{
    Function, Select, SqlColumn, SqlType, Update, call, case_when, col, int, rows_from, table, val,
};

impl TaskTransaction {
    pub async fn sibling_ids(
        &mut self,
        space: Option<&str>,
        parent: Option<&str>,
    ) -> Result<Vec<String>, TaskError> {
        Ok(Select::from("tasks")
            .columns([TaskColumn::Id])
            .where_(col(TaskColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(TaskColumn::SpaceId).not_distinct_from(val(space)))
            .where_(col(TaskColumn::ParentId).not_distinct_from(val(parent)))
            .order_by(col(TaskColumn::SortOrder).asc())
            .order_by(col(TaskColumn::Id).asc())
            .build()?
            .build_query_scalar()
            .fetch_all(&mut *self.tx)
            .await?)
    }
    pub async fn ranks(
        &mut self,
        ids: &[String],
        already_changed: Option<&str>,
    ) -> Result<(), TaskError> {
        let ranks = rows_from(call(Function::Unnest, [val(ids).cast(SqlType::TextArray)]))
            .with_ordinality()
            .alias("r")
            .columns([DerivedColumn::Id, DerivedColumn::Rank]);
        let unchanged = || TaskColumn::Id.of("t").eq(val(already_changed));
        Update::table(table("tasks").alias("t"))
            .set(TaskColumn::SortOrder.expression(DerivedColumn::Rank.of("r").minus(int(1))))
            .set(
                TaskColumn::Version.expression(
                    TaskColumn::Version
                        .of("t")
                        .plus(case_when([(unchanged(), int(0))], int(1))),
                ),
            )
            .set(TaskColumn::UpdatedAt.expression(case_when(
                [(unchanged(), TaskColumn::UpdatedAt.of("t"))],
                advance(TaskColumn::UpdatedAt.of("t")),
            )))
            .from(ranks)
            .where_(TaskColumn::OwnerId.of("t").eq(val(&self.owner)))
            .where_(TaskColumn::Id.of("t").eq(DerivedColumn::Id.of("r")))
            .where_(
                TaskColumn::SortOrder
                    .of("t")
                    .ne(DerivedColumn::Rank.of("r").minus(int(1))),
            )
            .build()?
            .build()
            .execute(&mut *self.tx)
            .await?;
        Ok(())
    }
}
