//! Task detail, filtered pages and batched aggregate hydration.
use super::super::{
    filter::{Scope, Sort, TaskPage, cursor_date, cursor_time},
    model::{Task, TaskError},
};
use super::{
    TaskTransaction,
    columns::{
        CommentColumn, DerivedColumn, LINK_COLUMNS, LinkColumn, TASK_COLUMNS, TagAssignmentColumn,
        TagColumn, TaskColumn,
    },
    rows::{AssignedTagRow, LinkRow, TaskRow},
};
use crate::adapters::postgres::query::{
    Direction, Expr, Function, Select, SqlColumn, all, call, case_when, col, int, table, tuple, val,
};

impl TaskTransaction {
    pub async fn task(&mut self, id: &str) -> Result<Task, TaskError> {
        let row = task_selection()
            .where_(TaskColumn::OwnerId.of("t").eq(val(&self.owner)))
            .where_(TaskColumn::Id.of("t").eq(val(id)))
            .build()?
            .build_query_as::<TaskRow>()
            .fetch_optional(&mut *self.tx)
            .await?
            .ok_or(TaskError::NotFound)?;
        let mut tasks = self.hydrate(vec![row]).await?;
        tasks.pop().ok_or(TaskError::Internal)
    }
    async fn hydrate(&mut self, rows: Vec<TaskRow>) -> Result<Vec<Task>, TaskError> {
        let mut tasks: Vec<Task> = rows.into_iter().map(Into::into).collect();
        if tasks.is_empty() {
            return Ok(tasks);
        }
        let task_ids: Vec<_> = tasks.iter().map(|t| t.id.clone()).collect();
        let tags = Select::from(table("task_tag_assignments").alias("a"))
            .columns([
                TagAssignmentColumn::TaskId.of("a"),
                TagColumn::Id.of("tag"),
                TagColumn::Name.of("tag"),
                TagColumn::CreatedAt.of("tag"),
            ])
            .join(
                table("task_tags").alias("tag"),
                TagColumn::OwnerId
                    .of("tag")
                    .eq(TagAssignmentColumn::OwnerId.of("a"))
                    .and(
                        TagColumn::Id
                            .of("tag")
                            .eq(TagAssignmentColumn::TagId.of("a")),
                    ),
            )
            .where_(TagAssignmentColumn::OwnerId.of("a").eq(val(&self.owner)))
            .where_(TagAssignmentColumn::TaskId.of("a").eq_any(val(&task_ids)))
            .order_by(TagColumn::NameKey.of("tag").asc())
            .order_by(TagColumn::Id.of("tag").asc())
            .build()?
            .build_query_as::<AssignedTagRow>()
            .fetch_all(&mut *self.tx)
            .await?;
        let links = Select::from("task_links")
            .columns(LINK_COLUMNS)
            .where_(col(LinkColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(LinkColumn::TaskId).eq_any(val(&task_ids)))
            .order_by(col(LinkColumn::SortOrder).asc())
            .order_by(col(LinkColumn::Id).asc())
            .build()?
            .build_query_as::<LinkRow>()
            .fetch_all(&mut *self.tx)
            .await?;
        let indices: std::collections::HashMap<_, _> = tasks
            .iter()
            .enumerate()
            .map(|(i, t)| (t.id.clone(), i))
            .collect();
        for tag in tags {
            if let Some(&i) = indices.get(&tag.task_id) {
                tasks[i].tags.push(tag.tag.into());
            }
        }
        for link in links {
            if let Some(&i) = indices.get(&link.task_id) {
                tasks[i].links.push(link.into());
            }
        }
        Ok(tasks)
    }
    pub async fn list(&mut self, page: &TaskPage) -> Result<Vec<Task>, TaskError> {
        let f = &page.filters;
        let mut query = task_selection().where_(TaskColumn::OwnerId.of("t").eq(val(&self.owner)));
        query = scope(query, TaskColumn::SpaceId, &f.space);
        query = scope(query, TaskColumn::ParentId, &f.parent);
        query = query.where_(TaskColumn::Status.of("t").eq_any(val(&f.statuses)));
        if let Some(priority) = &f.priority {
            query = query.where_(TaskColumn::Priority.of("t").eq(val(priority)));
        }
        if let Some(flagged) = f.flagged {
            query = query.where_(TaskColumn::Flagged.of("t").eq(val(flagged)));
        }
        if !f.tags.is_empty() {
            let assignments = Select::from(table("task_tag_assignments").alias("a"))
                .columns([if f.all_tags {
                    call(Function::Count, [all()])
                } else {
                    int(1)
                }])
                .where_(
                    TagAssignmentColumn::OwnerId
                        .of("a")
                        .eq(TaskColumn::OwnerId.of("t")),
                )
                .where_(
                    TagAssignmentColumn::TaskId
                        .of("a")
                        .eq(TaskColumn::Id.of("t")),
                )
                .where_(TagAssignmentColumn::TagId.of("a").eq_any(val(&f.tags)));
            query = query.where_(if f.all_tags {
                assignments.scalar().eq(val(f.tags.len() as i64))
            } else {
                assignments.exists()
            });
        }
        if let Some(has_due) = f.has_due {
            let due_on = || TaskColumn::DueOn.of("t");
            let due_at = || TaskColumn::DueAt.of("t");
            query = query.where_(if has_due {
                due_on().is_not_null().or(due_at().is_not_null())
            } else {
                due_on().is_null().and(due_at().is_null())
            });
        }
        if let Some(kind) = f.due_kind.as_deref() {
            let due = if kind == "date" {
                TaskColumn::DueOn
            } else {
                TaskColumn::DueAt
            };
            query = query.where_(due.of("t").is_not_null());
            for (value, inclusive) in [(&f.due_from, true), (&f.due_before, false)] {
                if let Some(value) = value {
                    let value = if kind == "date" {
                        val(super::super::model::date(value)?)
                    } else {
                        val(super::super::model::instant(value)?)
                    };
                    query = query.where_(if inclusive {
                        due.of("t").ge(value)
                    } else {
                        due.of("t").lt(value)
                    });
                }
            }
        }
        if let Some(from) = f.completed_from {
            query = query.where_(TaskColumn::CompletedAt.of("t").ge(val(from)));
        }
        if let Some(before) = f.completed_before {
            query = query.where_(TaskColumn::CompletedAt.of("t").lt(val(before)));
        }
        if let Some(cursor) = &page.cursor {
            let key = match f.sort {
                Sort::Created | Sort::DueTime => val(cursor_time(cursor.key)?),
                Sort::DueDate => val(cursor_date(cursor.key)?),
                _ => val(cursor.key),
            };
            let columns = tuple([sort_key(f.sort), TaskColumn::Id.of("t")]);
            let values = tuple([key, val(&cursor.id)]);
            query = query.where_(if f.sort == Sort::Created {
                columns.lt(values)
            } else {
                columns.gt(values)
            });
        }
        let direction = if f.sort == Sort::Created {
            Direction::Descending
        } else {
            Direction::Ascending
        };
        let rows = query
            .order_by(sort_key(f.sort).order(direction))
            .order_by(TaskColumn::Id.of("t").order(direction))
            .limit(val((page.limit + 1) as i64))
            .build()?
            .build_query_as::<TaskRow>()
            .fetch_all(&mut *self.tx)
            .await?;
        self.hydrate(rows).await
    }
}

/// Tasks aliased as `t`, projected as `TaskRow` with comment and child counts.
fn task_selection<'a>() -> Select<'a> {
    let comments = Select::from(table("task_comments").alias("c"))
        .columns([call(Function::Count, [all()])])
        .where_(
            CommentColumn::OwnerId
                .of("c")
                .eq(TaskColumn::OwnerId.of("t")),
        )
        .where_(CommentColumn::TaskId.of("c").eq(TaskColumn::Id.of("t")));
    let children = Select::from(table("tasks").alias("child"))
        .columns([call(Function::Count, [all()])])
        .where_(
            TaskColumn::OwnerId
                .of("child")
                .eq(TaskColumn::OwnerId.of("t")),
        )
        .where_(TaskColumn::ParentId.of("child").eq(TaskColumn::Id.of("t")));
    Select::from(table("tasks").alias("t"))
        .columns(TASK_COLUMNS.map(|column| column.of("t")))
        .columns([
            comments.scalar().alias(DerivedColumn::CommentCount),
            children.scalar().alias(DerivedColumn::ChildCount),
        ])
}

fn scope<'a>(query: Select<'a>, column: TaskColumn, scope: &'a Scope) -> Select<'a> {
    match scope {
        Scope::All => query,
        Scope::None => query.where_(column.of("t").is_null()),
        Scope::Id(id) => query.where_(column.of("t").eq(val(id))),
    }
}

fn sort_key<'a>(sort: Sort) -> Expr<'a> {
    match sort {
        Sort::Created => TaskColumn::CreatedAt.of("t"),
        Sort::DueDate => TaskColumn::DueOn.of("t"),
        Sort::DueTime => TaskColumn::DueAt.of("t"),
        Sort::Manual => TaskColumn::SortOrder.of("t"),
        Sort::Priority => {
            let priority = || TaskColumn::Priority.of("t");
            case_when(
                [
                    (priority().eq(val("high")), int(0)),
                    (priority().eq(val("medium")), int(1)),
                    (priority().eq(val("low")), int(2)),
                ],
                int(3),
            )
        }
    }
}
