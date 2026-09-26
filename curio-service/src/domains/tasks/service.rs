//! Task transitions and hierarchy rules, orchestrated inside owner-serialized transactions.
use super::{
    filter::{CreatedPage, ListQuery, PageQuery, Scope, TaskPage},
    model::*,
    repository::{TaskRepository, TaskTransaction},
};
use chrono::Utc;
use serde::Serialize;

pub struct TaskService {
    repository: TaskRepository,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksResponse {
    pub tasks: Vec<Task>,
    pub next_cursor: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagsResponse {
    pub tags: Vec<Tag>,
    pub next_cursor: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentsResponse {
    pub comments: Vec<Comment>,
    pub next_cursor: Option<String>,
}

impl TaskService {
    pub fn new(repository: TaskRepository) -> Self {
        Self { repository }
    }
    pub async fn create(&self, owner: &str, input: CreateTask) -> Result<Task, TaskError> {
        let tag_ids = ids(input.tag_ids)?;
        if input.links.len() > 50 {
            return Err(TaskError::Invalid("At most 50 links are allowed."));
        }
        let links = input
            .links
            .into_iter()
            .map(LinkInput::validate)
            .collect::<Result<Vec<_>, _>>()?;
        let now = Utc::now();
        let mut task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            title: text(&input.title, 240, false)?,
            description: optional_text(input.description, 10000, true)?,
            space_id: None,
            parent_id: input.parent_id.as_deref().map(id).transpose()?,
            status: input.status,
            priority: input.priority,
            flagged: input.flagged,
            due_on: input.due_on.as_deref().map(date).transpose()?,
            due_at: input.due_at.as_deref().map(instant).transpose()?,
            due_timezone: input.due_timezone,
            completed_at: None,
            sort_order: 0,
            version: 1,
            created_at: now,
            updated_at: now,
            tags: vec![],
            links: vec![],
            comment_count: 0,
            child_count: 0,
        };
        validate_task(&task)?;
        task.completed_at = completion("todo", &task.status, None, now);
        let mut tx = self.repository.write(owner, true).await?;
        let parent = if let Some(id) = &task.parent_id {
            Some(tx.task(id).await?)
        } else {
            None
        };
        task.space_id = match input.space_id {
            Field::Missing => parent.as_ref().and_then(|p| p.space_id.clone()),
            field => field.nullable(None).as_deref().map(id).transpose()?,
        };
        check_parent_space(&task, parent.as_ref())?;
        if let Some(parent) = parent {
            hierarchy(&task.id, &tx.ancestors(&parent.id).await?, 1)?;
        }
        references(&mut tx, &task, &tag_ids).await?;
        tx.insert(&task).await?;
        tx.replace_tags(&task.id, &tag_ids).await?;
        let mut link_ids = Vec::new();
        for input in links {
            let link = tx.create_link(&task.id, &input).await?;
            place(
                &mut link_ids,
                &link.id,
                input.sort_order.map(order).transpose()?,
            );
        }
        tx.link_ranks(&task.id, &link_ids).await?;
        reorder(&mut tx, &task, None).await?;
        let result = tx.task(&task.id).await?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn get(&self, owner: &str, id: &str) -> Result<Task, TaskError> {
        let mut tx = self.repository.read(owner).await?;
        let result = tx.task(id).await?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn list(&self, owner: &str, query: ListQuery) -> Result<TasksResponse, TaskError> {
        let page = TaskPage::parse(query)?;
        let mut tx = self.repository.read(owner).await?;
        if let Scope::Id(space) = &page.filters.space {
            tx.space(space).await?;
        }
        if let Scope::Id(parent) = &page.filters.parent {
            tx.task(parent).await?;
        }
        tx.check_tags(&page.filters.tags).await?;
        let mut tasks = tx.list(&page).await?;
        let next_cursor = if tasks.len() > page.limit {
            tasks.truncate(page.limit);
            tasks.last().map(|t| page.next(t)).transpose()?
        } else {
            None
        };
        tx.commit().await?;
        Ok(TasksResponse { tasks, next_cursor })
    }
    pub async fn patch(&self, owner: &str, id: &str, patch: PatchTask) -> Result<Task, TaskError> {
        patch.validate()?;
        let mut tx = self.repository.write(owner, false).await?;
        let old = tx.task(id).await?;
        if old.version != patch.expected_version {
            return Err(TaskError::Conflict);
        }
        let position = match patch.sort_order {
            Field::Missing => None,
            Field::Null => return Err(TaskError::Invalid("sortOrder cannot be null.")),
            Field::Value(v) => Some(order(v)?),
        };
        let tag_ids = match patch.tag_ids {
            Field::Missing => None,
            Field::Null => return Err(TaskError::Invalid("tagIds cannot be null.")),
            Field::Value(v) => Some(ids(v)?),
        };
        let mut task = old.clone();
        task.title = text(&patch.title.required(task.title)?, 240, false)?;
        task.description =
            optional_text(patch.description.nullable(task.description), 10000, true)?;
        task.status = patch.status.required(task.status)?;
        task.priority = patch.priority.nullable(task.priority);
        task.flagged = patch.flagged.required(task.flagged)?;
        task.parent_id = patch
            .parent_id
            .nullable(task.parent_id)
            .as_deref()
            .map(super::model::id)
            .transpose()?;
        task.due_on = patch
            .due_on
            .nullable(task.due_on.map(|v| v.to_string()))
            .as_deref()
            .map(date)
            .transpose()?;
        task.due_at = patch
            .due_at
            .nullable(task.due_at.map(|v| v.to_rfc3339()))
            .as_deref()
            .map(instant)
            .transpose()?;
        task.due_timezone = patch.due_timezone.nullable(task.due_timezone);
        validate_task(&task)?;
        task.completed_at = completion(&old.status, &task.status, old.completed_at, Utc::now());
        let parent = if let Some(id) = &task.parent_id {
            Some(tx.task(id).await?)
        } else {
            None
        };
        task.space_id = match patch.space_id {
            Field::Missing if old.parent_id != task.parent_id && parent.is_some() => {
                parent.as_ref().and_then(|p| p.space_id.clone())
            }
            field => field
                .nullable(task.space_id)
                .as_deref()
                .map(super::model::id)
                .transpose()?,
        };
        check_parent_space(&task, parent.as_ref())?;
        let subtree = tx.descendants(id).await?;
        let ancestors = if let Some(parent) = parent {
            tx.ancestors(&parent.id).await?
        } else {
            vec![]
        };
        hierarchy(
            id,
            &ancestors,
            subtree.iter().map(|n| n.depth).max().unwrap_or(1),
        )?;
        references(&mut tx, &task, tag_ids.as_deref().unwrap_or(&[])).await?;
        tx.update(&task, patch.expected_version).await?;
        if let Some(tags) = tag_ids {
            tx.replace_tags(id, &tags).await?;
        }
        if old.space_id != task.space_id {
            let descendants: Vec<_> = subtree
                .into_iter()
                .filter(|n| n.id != id)
                .map(|n| n.id)
                .collect();
            tx.move_descendants(&descendants, task.space_id.as_deref())
                .await?;
        }
        if old.space_id != task.space_id || old.parent_id != task.parent_id {
            let previous = tx
                .sibling_ids(old.space_id.as_deref(), old.parent_id.as_deref())
                .await?;
            tx.ranks(&previous, None).await?;
            reorder(&mut tx, &task, position).await?;
        } else if position.is_some() {
            reorder(&mut tx, &task, position).await?;
        }
        let result = tx.task(id).await?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn delete(&self, owner: &str, id: &str, expected: i64) -> Result<(), TaskError> {
        version(expected)?;
        let mut tx = self.repository.write(owner, false).await?;
        let task = tx.task(id).await?;
        if task.version != expected {
            return Err(TaskError::Conflict);
        }
        let descendants: Vec<_> = tx
            .descendants(id)
            .await?
            .into_iter()
            .map(|n| n.id)
            .collect();
        tx.delete(id, expected, &descendants).await?;
        let remaining = tx
            .sibling_ids(task.space_id.as_deref(), task.parent_id.as_deref())
            .await?;
        tx.ranks(&remaining, None).await?;
        tx.commit().await
    }
    pub async fn tags(&self, owner: &str, query: PageQuery) -> Result<TagsResponse, TaskError> {
        let page = CreatedPage::parse(query, "tags".into())?;
        let mut tx = self.repository.read(owner).await?;
        let mut tags = tx.tags(&page).await?;
        let next_cursor = if tags.len() > page.limit {
            tags.truncate(page.limit);
            tags.last()
                .map(|t| page.next(t.created_at, &t.id))
                .transpose()?
        } else {
            None
        };
        tx.commit().await?;
        Ok(TagsResponse { tags, next_cursor })
    }
    pub async fn create_tag(&self, owner: &str, input: TagInput) -> Result<Tag, TaskError> {
        let (name, key) = tag_name(&input.name)?;
        let mut tx = self.repository.write(owner, true).await?;
        let tag = tx.create_tag(&name, &key).await?;
        tx.commit().await?;
        Ok(tag)
    }
    pub async fn rename_tag(
        &self,
        owner: &str,
        id: &str,
        input: TagInput,
    ) -> Result<Tag, TaskError> {
        let (name, key) = tag_name(&input.name)?;
        let mut tx = self.repository.write(owner, false).await?;
        let tag = tx.rename_tag(id, &name, &key).await?;
        tx.commit().await?;
        Ok(tag)
    }
    pub async fn delete_tag(&self, owner: &str, id: &str) -> Result<(), TaskError> {
        let mut tx = self.repository.write(owner, false).await?;
        tx.delete_tag(id).await?;
        tx.commit().await
    }
    pub async fn comments(
        &self,
        owner: &str,
        task: &str,
        query: PageQuery,
    ) -> Result<CommentsResponse, TaskError> {
        let page = CreatedPage::parse(query, format!("comments:{task}"))?;
        let mut tx = self.repository.read(owner).await?;
        tx.task(task).await?;
        let mut comments = tx.comments(task, &page).await?;
        let next_cursor = if comments.len() > page.limit {
            comments.truncate(page.limit);
            comments
                .last()
                .map(|c| page.next(c.created_at, &c.id))
                .transpose()?
        } else {
            None
        };
        tx.commit().await?;
        Ok(CommentsResponse {
            comments,
            next_cursor,
        })
    }
    pub async fn create_comment(
        &self,
        owner: &str,
        task: &str,
        input: CommentInput,
    ) -> Result<Comment, TaskError> {
        let body = text(&input.body, 10000, true)?;
        let mut tx = self.repository.write(owner, false).await?;
        let result = tx.create_comment(task, &body).await?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn edit_comment(
        &self,
        owner: &str,
        task: &str,
        comment: &str,
        input: CommentInput,
    ) -> Result<Comment, TaskError> {
        let body = text(&input.body, 10000, true)?;
        let mut tx = self.repository.write(owner, false).await?;
        let result = tx.edit_comment(task, comment, &body).await?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn delete_comment(
        &self,
        owner: &str,
        task: &str,
        comment: &str,
    ) -> Result<(), TaskError> {
        let mut tx = self.repository.write(owner, false).await?;
        tx.delete_comment(task, comment).await?;
        tx.commit().await
    }
    pub async fn create_link(
        &self,
        owner: &str,
        task: &str,
        input: LinkInput,
    ) -> Result<Link, TaskError> {
        let input = input.validate()?;
        let mut tx = self.repository.write(owner, false).await?;
        tx.task(task).await?;
        let mut ids = tx.link_ids(task).await?;
        if ids.len() >= 50 {
            return Err(TaskError::Invalid("At most 50 links are allowed."));
        }
        let link = tx.create_link(task, &input).await?;
        place(&mut ids, &link.id, input.sort_order.map(order).transpose()?);
        tx.link_ranks(task, &ids).await?;
        let result = tx.link(task, &link.id).await?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn edit_link(
        &self,
        owner: &str,
        task: &str,
        link: &str,
        patch: PatchLink,
    ) -> Result<Link, TaskError> {
        let mut tx = self.repository.write(owner, false).await?;
        let old = tx.link(task, link).await?;
        let input = patch.merge(old)?;
        tx.edit_link(task, link, &input).await?;
        let mut ids = tx.link_ids(task).await?;
        place(&mut ids, link, input.sort_order.map(order).transpose()?);
        tx.link_ranks(task, &ids).await?;
        let result = tx.link(task, link).await?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn delete_link(&self, owner: &str, task: &str, link: &str) -> Result<(), TaskError> {
        let mut tx = self.repository.write(owner, false).await?;
        tx.delete_link(task, link).await?;
        let ids = tx.link_ids(task).await?;
        tx.link_ranks(task, &ids).await?;
        tx.commit().await
    }
}
fn validate_task(task: &Task) -> Result<(), TaskError> {
    status(&task.status)?;
    priority(task.priority.as_deref())?;
    due(task.due_on, task.due_at, task.due_timezone.as_deref())
}
fn check_parent_space(task: &Task, parent: Option<&Task>) -> Result<(), TaskError> {
    if parent.is_some_and(|p| p.space_id != task.space_id) {
        Err(TaskError::Invalid(
            "A child must belong to its parent's Space.",
        ))
    } else {
        Ok(())
    }
}
async fn references(
    tx: &mut TaskTransaction,
    task: &Task,
    tags: &[String],
) -> Result<(), TaskError> {
    if let Some(space) = &task.space_id {
        tx.space(space).await?;
    }
    if let Some(zone) = &task.due_timezone {
        tx.timezone(zone).await?;
    }
    tx.check_tags(tags).await
}
/// A requested rank is an insertion position, clamped to the list's end.
pub fn place(ids: &mut Vec<String>, id: &str, position: Option<usize>) {
    ids.retain(|existing| existing != id);
    ids.insert(position.unwrap_or(ids.len()).min(ids.len()), id.to_owned());
}
async fn reorder(
    tx: &mut TaskTransaction,
    task: &Task,
    position: Option<usize>,
) -> Result<(), TaskError> {
    let mut ids = tx
        .sibling_ids(task.space_id.as_deref(), task.parent_id.as_deref())
        .await?;
    place(&mut ids, &task.id, position);
    tx.ranks(&ids, Some(&task.id)).await
}
