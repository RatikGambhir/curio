//! Ancestor/descendant traversal and Space membership movement.
use super::super::model::{TaskError, TreeNode};
use super::{
    TaskTransaction,
    columns::{DerivedColumn, TaskColumn},
    queries::advance,
    rows::TreeRow,
};
use crate::adapters::postgres::query::{
    Function, Select, SqlColumn, Update, call, col, int, null, table, val,
};
use sqlx::{Postgres, Transaction};

impl TaskTransaction {
    pub async fn ancestors(&mut self, parent: &str) -> Result<Vec<TreeNode>, TaskError> {
        let chain = Select::from("tasks")
            .columns([
                col(TaskColumn::Id),
                col(TaskColumn::ParentId),
                int(1).alias(DerivedColumn::Depth),
            ])
            .where_(col(TaskColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(TaskColumn::Id).eq(val(parent)))
            .union_all(
                Select::from(table("tasks").alias("t"))
                    .columns([
                        TaskColumn::Id.of("t"),
                        TaskColumn::ParentId.of("t"),
                        DerivedColumn::Depth.of("c").plus(int(1)),
                    ])
                    .join(
                        table("chain").alias("c"),
                        TaskColumn::Id.of("t").eq(TaskColumn::ParentId.of("c")),
                    )
                    .where_(TaskColumn::OwnerId.of("t").eq(val(&self.owner)))
                    .where_(DerivedColumn::Depth.of("c").lt(int(6))),
            );
        let rows = Select::from("chain")
            .columns([col(TaskColumn::Id), col(DerivedColumn::Depth)])
            .with_recursive("chain", chain)
            .build()?
            .build_query_as::<TreeRow>()
            .fetch_all(&mut *self.tx)
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| TreeNode {
                id: r.id,
                depth: r.depth,
            })
            .collect())
    }
    pub async fn descendants(&mut self, id: &str) -> Result<Vec<TreeNode>, TaskError> {
        let tree = Select::from("tasks")
            .columns([col(TaskColumn::Id), int(1).alias(DerivedColumn::Depth)])
            .where_(col(TaskColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(TaskColumn::Id).eq(val(id)))
            .union_all(
                Select::from(table("tasks").alias("t"))
                    .columns([
                        TaskColumn::Id.of("t"),
                        DerivedColumn::Depth.of("tree").plus(int(1)),
                    ])
                    .join(
                        "tree",
                        TaskColumn::ParentId.of("t").eq(TaskColumn::Id.of("tree")),
                    )
                    .where_(TaskColumn::OwnerId.of("t").eq(val(&self.owner)))
                    .where_(DerivedColumn::Depth.of("tree").lt(int(6))),
            );
        let rows = Select::from("tree")
            .columns([col(TaskColumn::Id), col(DerivedColumn::Depth)])
            .with_recursive("tree", tree)
            .build()?
            .build_query_as::<TreeRow>()
            .fetch_all(&mut *self.tx)
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| TreeNode {
                id: r.id,
                depth: r.depth,
            })
            .collect())
    }
    pub async fn move_descendants(
        &mut self,
        ids: &[String],
        space: Option<&str>,
    ) -> Result<(), TaskError> {
        Update::table("tasks")
            .set(TaskColumn::SpaceId.value(space))
            .set(TaskColumn::Version.expression(col(TaskColumn::Version).plus(int(1))))
            .set(TaskColumn::UpdatedAt.expression(advance(col(TaskColumn::UpdatedAt))))
            .where_(col(TaskColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(TaskColumn::Id).eq_any(val(ids)))
            .where_(col(TaskColumn::SpaceId).distinct_from(val(space)))
            .build()?
            .build()
            .execute(&mut *self.tx)
            .await?;
        Ok(())
    }
}

/// Detach membership and normalize merged sibling lists after the shared owner lock.
pub(crate) async fn detach_space_tasks(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
    space: &str,
) -> Result<(), sqlx::Error> {
    let ranks = Select::from("tasks")
        .columns([
            col(TaskColumn::Id),
            call(Function::RowNumber, [])
                .over(
                    [col(TaskColumn::ParentId)],
                    [col(TaskColumn::SortOrder).asc(), col(TaskColumn::Id).asc()],
                )
                .minus(int(1))
                .alias(DerivedColumn::Rank),
        ])
        .where_(col(TaskColumn::OwnerId).eq(val(owner)))
        .where_(
            col(TaskColumn::SpaceId)
                .eq(val(space))
                .or(col(TaskColumn::SpaceId).is_null()),
        );
    Update::table(table("tasks").alias("t"))
        .with("ranks", ranks)
        .set(TaskColumn::SpaceId.expression(null()))
        .set(TaskColumn::SortOrder.expression(DerivedColumn::Rank.of("r")))
        .set(TaskColumn::Version.expression(TaskColumn::Version.of("t").plus(int(1))))
        .set(TaskColumn::UpdatedAt.expression(advance(TaskColumn::UpdatedAt.of("t"))))
        .from(table("ranks").alias("r"))
        .where_(TaskColumn::OwnerId.of("t").eq(val(owner)))
        .where_(TaskColumn::Id.of("t").eq(DerivedColumn::Id.of("r")))
        .where_(
            TaskColumn::SpaceId
                .of("t")
                .is_not_null()
                .or(TaskColumn::SortOrder
                    .of("t")
                    .ne(DerivedColumn::Rank.of("r"))),
        )
        .build()?
        .build()
        .execute(&mut **tx)
        .await?;
    Ok(())
}
