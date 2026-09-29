//! SQLx result rows and conversion into task-domain records.
use super::super::model::{Comment, Link, Tag, Task};
use chrono::{DateTime, NaiveDate, Utc};

#[derive(sqlx::FromRow)]
pub(super) struct TaskRow {
    pub(super) id: String,
    pub(super) space_id: Option<String>,
    pub(super) parent_id: Option<String>,
    pub(super) title: String,
    pub(super) description: Option<String>,
    pub(super) status: String,
    pub(super) priority: Option<String>,
    pub(super) flagged: bool,
    pub(super) due_on: Option<NaiveDate>,
    pub(super) due_at: Option<DateTime<Utc>>,
    pub(super) due_timezone: Option<String>,
    pub(super) completed_at: Option<DateTime<Utc>>,
    pub(super) sort_order: i64,
    pub(super) version: i64,
    pub(super) created_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
    pub(super) comment_count: i64,
    pub(super) child_count: i64,
}
impl From<TaskRow> for Task {
    fn from(r: TaskRow) -> Self {
        Self {
            id: r.id,
            space_id: r.space_id,
            parent_id: r.parent_id,
            title: r.title,
            description: r.description,
            status: r.status,
            priority: r.priority,
            flagged: r.flagged,
            due_on: r.due_on,
            due_at: r.due_at,
            due_timezone: r.due_timezone,
            completed_at: r.completed_at,
            sort_order: r.sort_order,
            version: r.version,
            created_at: r.created_at,
            updated_at: r.updated_at,
            comment_count: r.comment_count,
            child_count: r.child_count,
            tags: vec![],
            links: vec![],
        }
    }
}
#[derive(sqlx::FromRow)]
pub(super) struct TreeRow {
    pub(super) id: String,
    pub(super) depth: i32,
}
#[derive(sqlx::FromRow)]
pub(super) struct TagRow {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) created_at: DateTime<Utc>,
}
impl From<TagRow> for Tag {
    fn from(r: TagRow) -> Self {
        Self {
            id: r.id,
            name: r.name,
            created_at: r.created_at,
        }
    }
}
#[derive(sqlx::FromRow)]
pub(super) struct AssignedTagRow {
    pub(super) task_id: String,
    #[sqlx(flatten)]
    pub(super) tag: TagRow,
}
#[derive(sqlx::FromRow)]
pub(super) struct CommentRow {
    pub(super) id: String,
    pub(super) task_id: String,
    pub(super) author_id: String,
    pub(super) body: String,
    pub(super) created_at: DateTime<Utc>,
    pub(super) edited_at: Option<DateTime<Utc>>,
}
impl From<CommentRow> for Comment {
    fn from(r: CommentRow) -> Self {
        Self {
            id: r.id,
            task_id: r.task_id,
            author_id: r.author_id,
            body: r.body,
            created_at: r.created_at,
            edited_at: r.edited_at,
        }
    }
}
#[derive(sqlx::FromRow)]
pub(super) struct LinkRow {
    pub(super) id: String,
    pub(super) task_id: String,
    pub(super) kind: String,
    pub(super) label: Option<String>,
    pub(super) url: Option<String>,
    pub(super) note_id: Option<String>,
    pub(super) sort_order: i64,
    pub(super) created_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
}
impl From<LinkRow> for Link {
    fn from(r: LinkRow) -> Self {
        Self {
            id: r.id,
            task_id: r.task_id,
            kind: r.kind,
            label: r.label,
            url: r.url,
            note_id: r.note_id,
            sort_order: r.sort_order,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
