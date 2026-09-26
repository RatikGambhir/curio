//! Owner-scoped SQL, transaction lifecycle and aggregate hydration.
use super::{
    filter::{CreatedPage, Scope, Sort, TaskPage, cursor_date, cursor_time},
    model::*,
};
use crate::adapters::postgres::client::Database;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{Postgres, QueryBuilder, Transaction};

const TASK_SELECT: &str = "SELECT t.*, (SELECT count(*) FROM task_comments c WHERE c.owner_id=t.owner_id AND c.task_id=t.id) AS comment_count, (SELECT count(*) FROM tasks child WHERE child.owner_id=t.owner_id AND child.parent_id=t.id) AS child_count FROM tasks t";
const ADVANCE: &str =
    "GREATEST(clock_timestamp()::timestamptz(3), updated_at + interval '1 millisecond')";

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
        let found = sqlx::query_scalar::<_, String>("SELECT id FROM users WHERE id=$1 FOR UPDATE")
            .bind(owner)
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
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await?;
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
    pub async fn task(&mut self, id: &str) -> Result<Task, TaskError> {
        let row =
            sqlx::query_as::<_, TaskRow>(&format!("{TASK_SELECT} WHERE t.owner_id=$1 AND t.id=$2"))
                .bind(&self.owner)
                .bind(id)
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
        let tags = sqlx::query_as::<_, AssignedTagRow>("SELECT a.task_id, tag.id, tag.name, tag.created_at FROM task_tag_assignments a JOIN task_tags tag ON tag.owner_id=a.owner_id AND tag.id=a.tag_id WHERE a.owner_id=$1 AND a.task_id=ANY($2) ORDER BY tag.name_key, tag.id")
            .bind(&self.owner).bind(&task_ids).fetch_all(&mut *self.tx).await?;
        let links = sqlx::query_as::<_, LinkRow>(
            "SELECT * FROM task_links WHERE owner_id=$1 AND task_id=ANY($2) ORDER BY sort_order,id",
        )
        .bind(&self.owner)
        .bind(&task_ids)
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
        let mut q = QueryBuilder::<Postgres>::new(TASK_SELECT);
        q.push(" WHERE t.owner_id=").push_bind(&self.owner);
        scope(&mut q, "t.space_id", &f.space);
        scope(&mut q, "t.parent_id", &f.parent);
        q.push(" AND t.status=ANY(")
            .push_bind(&f.statuses)
            .push(")");
        if let Some(priority) = &f.priority {
            q.push(" AND t.priority=").push_bind(priority);
        }
        if let Some(flagged) = f.flagged {
            q.push(" AND t.flagged=").push_bind(flagged);
        }
        if !f.tags.is_empty() {
            if f.all_tags {
                q.push(" AND (SELECT count(*) FROM task_tag_assignments a WHERE a.owner_id=t.owner_id AND a.task_id=t.id AND a.tag_id=ANY(").push_bind(&f.tags).push("))=").push_bind(f.tags.len() as i64);
            } else {
                q.push(" AND EXISTS(SELECT 1 FROM task_tag_assignments a WHERE a.owner_id=t.owner_id AND a.task_id=t.id AND a.tag_id=ANY(").push_bind(&f.tags).push("))");
            }
        }
        if let Some(has_due) = f.has_due {
            q.push(if has_due {
                " AND (t.due_on IS NOT NULL OR t.due_at IS NOT NULL)"
            } else {
                " AND t.due_on IS NULL AND t.due_at IS NULL"
            });
        }
        if let Some(kind) = f.due_kind.as_deref() {
            q.push(if kind == "date" {
                " AND t.due_on IS NOT NULL"
            } else {
                " AND t.due_at IS NOT NULL"
            });
            for (value, op) in [(&f.due_from, ">="), (&f.due_before, "<")] {
                if let Some(value) = value {
                    q.push(if kind == "date" {
                        " AND t.due_on"
                    } else {
                        " AND t.due_at"
                    })
                    .push(op);
                    if kind == "date" {
                        q.push_bind(super::model::date(value)?);
                    } else {
                        q.push_bind(super::model::instant(value)?);
                    }
                }
            }
        }
        if let Some(from) = f.completed_from {
            q.push(" AND t.completed_at>=").push_bind(from);
        }
        if let Some(before) = f.completed_before {
            q.push(" AND t.completed_at<").push_bind(before);
        }
        let expression = match f.sort {
            Sort::Created => "t.created_at",
            Sort::DueDate => "t.due_on",
            Sort::DueTime => "t.due_at",
            Sort::Manual => "t.sort_order",
            Sort::Priority => {
                "(CASE t.priority WHEN 'high' THEN 0 WHEN 'medium' THEN 1 WHEN 'low' THEN 2 ELSE 3 END)"
            }
        };
        if let Some(cursor) = &page.cursor {
            q.push(" AND (")
                .push(expression)
                .push(",t.id)")
                .push(if f.sort == Sort::Created { "<(" } else { ">(" });
            match f.sort {
                Sort::Created | Sort::DueTime => {
                    q.push_bind(cursor_time(cursor.key)?);
                }
                Sort::DueDate => {
                    q.push_bind(cursor_date(cursor.key)?);
                }
                _ => {
                    q.push_bind(cursor.key);
                }
            }
            q.push(",").push_bind(&cursor.id).push(")");
        }
        q.push(" ORDER BY ")
            .push(expression)
            .push(if f.sort == Sort::Created {
                " DESC,t.id DESC"
            } else {
                " ASC,t.id ASC"
            });
        q.push(" LIMIT ").push_bind((page.limit + 1) as i64);
        let rows = q
            .build_query_as::<TaskRow>()
            .fetch_all(&mut *self.tx)
            .await?;
        self.hydrate(rows).await
    }
    pub async fn space(&mut self, id: &str) -> Result<(), TaskError> {
        sqlx::query_scalar::<_, String>("SELECT id FROM spaces WHERE owner_id=$1 AND id=$2")
            .bind(&self.owner)
            .bind(id)
            .fetch_optional(&mut *self.tx)
            .await?
            .ok_or(TaskError::NotFound)
            .map(|_| ())
    }
    pub async fn timezone(&mut self, zone: &str) -> Result<(), TaskError> {
        let valid: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_timezone_names WHERE name=$1)",
        )
        .bind(zone)
        .fetch_one(&mut *self.tx)
        .await?;
        if valid {
            Ok(())
        } else {
            Err(TaskError::Invalid("Unknown IANA timezone."))
        }
    }
    pub async fn check_tags(&mut self, tags: &[String]) -> Result<(), TaskError> {
        let found: i64 =
            sqlx::query_scalar("SELECT count(*) FROM task_tags WHERE owner_id=$1 AND id=ANY($2)")
                .bind(&self.owner)
                .bind(tags)
                .fetch_one(&mut *self.tx)
                .await?;
        if found == tags.len() as i64 {
            Ok(())
        } else {
            Err(TaskError::NotFound)
        }
    }
    pub async fn ancestors(&mut self, parent: &str) -> Result<Vec<TreeNode>, TaskError> {
        let rows = sqlx::query_as::<_, TreeRow>("WITH RECURSIVE chain AS (SELECT id,parent_id,1 AS depth FROM tasks WHERE owner_id=$1 AND id=$2 UNION ALL SELECT t.id,t.parent_id,c.depth+1 FROM tasks t JOIN chain c ON t.id=c.parent_id WHERE t.owner_id=$1 AND c.depth<6) SELECT id,depth FROM chain")
            .bind(&self.owner).bind(parent).fetch_all(&mut *self.tx).await?;
        Ok(rows
            .into_iter()
            .map(|r| TreeNode {
                id: r.id,
                depth: r.depth,
            })
            .collect())
    }
    pub async fn descendants(&mut self, id: &str) -> Result<Vec<TreeNode>, TaskError> {
        let rows = sqlx::query_as::<_, TreeRow>("WITH RECURSIVE tree AS (SELECT id,1 AS depth FROM tasks WHERE owner_id=$1 AND id=$2 UNION ALL SELECT t.id,tree.depth+1 FROM tasks t JOIN tree ON t.parent_id=tree.id WHERE t.owner_id=$1 AND tree.depth<6) SELECT id,depth FROM tree")
            .bind(&self.owner).bind(id).fetch_all(&mut *self.tx).await?;
        Ok(rows
            .into_iter()
            .map(|r| TreeNode {
                id: r.id,
                depth: r.depth,
            })
            .collect())
    }
    pub async fn insert(&mut self, task: &Task) -> Result<(), TaskError> {
        sqlx::query("INSERT INTO tasks (id,owner_id,space_id,parent_id,title,description,status,priority,flagged,due_on,due_at,due_timezone,completed_at,sort_order) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14) RETURNING id")
            .bind(&task.id).bind(&self.owner).bind(&task.space_id).bind(&task.parent_id).bind(&task.title).bind(&task.description)
            .bind(&task.status).bind(&task.priority).bind(task.flagged).bind(task.due_on).bind(task.due_at).bind(&task.due_timezone).bind(task.completed_at).bind(task.sort_order)
            .fetch_one(&mut *self.tx).await?;
        Ok(())
    }
    pub async fn update(&mut self, task: &Task, expected: i64) -> Result<(), TaskError> {
        let found = sqlx::query_scalar::<_, String>(&format!("UPDATE tasks SET space_id=$3,parent_id=$4,title=$5,description=$6,status=$7,priority=$8,flagged=$9,due_on=$10,due_at=$11,due_timezone=$12,completed_at=$13,version=version+1,updated_at={ADVANCE} WHERE owner_id=$1 AND id=$2 AND version=$14 RETURNING id"))
            .bind(&self.owner).bind(&task.id).bind(&task.space_id).bind(&task.parent_id).bind(&task.title).bind(&task.description)
            .bind(&task.status).bind(&task.priority).bind(task.flagged).bind(task.due_on).bind(task.due_at).bind(&task.due_timezone).bind(task.completed_at).bind(expected)
            .fetch_optional(&mut *self.tx).await?;
        found.ok_or(TaskError::Conflict).map(|_| ())
    }
    pub async fn replace_tags(&mut self, task: &str, tags: &[String]) -> Result<(), TaskError> {
        sqlx::query("DELETE FROM task_tag_assignments WHERE owner_id=$1 AND task_id=$2")
            .bind(&self.owner)
            .bind(task)
            .execute(&mut *self.tx)
            .await?;
        sqlx::query("INSERT INTO task_tag_assignments (owner_id,task_id,tag_id) SELECT $1,$2,unnest($3::text[])").bind(&self.owner).bind(task).bind(tags).execute(&mut *self.tx).await?;
        Ok(())
    }
    pub async fn move_descendants(
        &mut self,
        ids: &[String],
        space: Option<&str>,
    ) -> Result<(), TaskError> {
        sqlx::query(&format!("UPDATE tasks SET space_id=$3,version=version+1,updated_at={ADVANCE} WHERE owner_id=$1 AND id=ANY($2) AND space_id IS DISTINCT FROM $3"))
            .bind(&self.owner).bind(ids).bind(space).execute(&mut *self.tx).await?;
        Ok(())
    }
    pub async fn sibling_ids(
        &mut self,
        space: Option<&str>,
        parent: Option<&str>,
    ) -> Result<Vec<String>, TaskError> {
        Ok(sqlx::query_scalar("SELECT id FROM tasks WHERE owner_id=$1 AND space_id IS NOT DISTINCT FROM $2 AND parent_id IS NOT DISTINCT FROM $3 ORDER BY sort_order,id")
            .bind(&self.owner).bind(space).bind(parent).fetch_all(&mut *self.tx).await?)
    }
    pub async fn ranks(
        &mut self,
        ids: &[String],
        already_changed: Option<&str>,
    ) -> Result<(), TaskError> {
        sqlx::query("UPDATE tasks t SET sort_order=r.rank-1,version=t.version+CASE WHEN t.id=$3 THEN 0 ELSE 1 END,updated_at=CASE WHEN t.id=$3 THEN t.updated_at ELSE GREATEST(clock_timestamp()::timestamptz(3),t.updated_at+interval '1 millisecond') END FROM unnest($2::text[]) WITH ORDINALITY AS r(id,rank) WHERE t.owner_id=$1 AND t.id=r.id AND t.sort_order<>r.rank-1")
            .bind(&self.owner).bind(ids).bind(already_changed).execute(&mut *self.tx).await?;
        Ok(())
    }
    pub async fn delete(
        &mut self,
        task: &str,
        expected: i64,
        descendants: &[String],
    ) -> Result<(), TaskError> {
        // Delete one subtree statement, with the root version in the same write predicate.
        let deleted: Vec<String> = sqlx::query_scalar("DELETE FROM tasks WHERE owner_id=$1 AND id=ANY($2) AND EXISTS(SELECT 1 FROM tasks root WHERE root.owner_id=$1 AND root.id=$3 AND root.version=$4) RETURNING id")
            .bind(&self.owner).bind(descendants).bind(task).bind(expected).fetch_all(&mut *self.tx).await?;
        if deleted.is_empty() {
            Err(TaskError::Conflict)
        } else {
            Ok(())
        }
    }

    pub async fn tags(&mut self, page: &CreatedPage) -> Result<Vec<Tag>, TaskError> {
        let timestamp = page
            .cursor
            .as_ref()
            .map(|c| cursor_time(c.created))
            .transpose()?;
        let rows = sqlx::query_as::<_, TagRow>("SELECT * FROM task_tags WHERE owner_id=$1 AND ($2::timestamptz IS NULL OR (created_at,id)<($2,$3)) ORDER BY created_at DESC,id DESC LIMIT $4")
            .bind(&self.owner).bind(timestamp).bind(page.cursor.as_ref().map(|c| &c.id)).bind((page.limit+1) as i64).fetch_all(&mut *self.tx).await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
    pub async fn create_tag(&mut self, name: &str, key: &str) -> Result<Tag, TaskError> {
        Ok(sqlx::query_as::<_, TagRow>(
            "INSERT INTO task_tags (id,owner_id,name,name_key) VALUES ($1,$2,$3,$4) RETURNING *",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&self.owner)
        .bind(name)
        .bind(key)
        .fetch_one(&mut *self.tx)
        .await?
        .into())
    }
    pub async fn rename_tag(&mut self, id: &str, name: &str, key: &str) -> Result<Tag, TaskError> {
        Ok(sqlx::query_as::<_, TagRow>(
            "UPDATE task_tags SET name=$3,name_key=$4 WHERE owner_id=$1 AND id=$2 RETURNING *",
        )
        .bind(&self.owner)
        .bind(id)
        .bind(name)
        .bind(key)
        .fetch_optional(&mut *self.tx)
        .await?
        .ok_or(TaskError::NotFound)?
        .into())
    }
    pub async fn delete_tag(&mut self, id: &str) -> Result<(), TaskError> {
        // Removing assignments changes the aggregate protected by expectedVersion.
        sqlx::query(&format!("UPDATE tasks SET version=version+1,updated_at={ADVANCE} WHERE owner_id=$1 AND id IN (SELECT task_id FROM task_tag_assignments WHERE owner_id=$1 AND tag_id=$2)"))
            .bind(&self.owner).bind(id).execute(&mut *self.tx).await?;
        sqlx::query_scalar::<_, String>(
            "DELETE FROM task_tags WHERE owner_id=$1 AND id=$2 RETURNING id",
        )
        .bind(&self.owner)
        .bind(id)
        .fetch_optional(&mut *self.tx)
        .await?
        .ok_or(TaskError::NotFound)
        .map(|_| ())
    }
    pub async fn comments(
        &mut self,
        task: &str,
        page: &CreatedPage,
    ) -> Result<Vec<Comment>, TaskError> {
        let timestamp = page
            .cursor
            .as_ref()
            .map(|c| cursor_time(c.created))
            .transpose()?;
        let rows = sqlx::query_as::<_, CommentRow>("SELECT c.* FROM task_comments c JOIN tasks t ON t.owner_id=c.owner_id AND t.id=c.task_id WHERE c.owner_id=$1 AND c.task_id=$2 AND ($3::timestamptz IS NULL OR (c.created_at,c.id)<($3,$4)) ORDER BY c.created_at DESC,c.id DESC LIMIT $5")
            .bind(&self.owner).bind(task).bind(timestamp).bind(page.cursor.as_ref().map(|c| &c.id)).bind((page.limit+1) as i64).fetch_all(&mut *self.tx).await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
    pub async fn create_comment(&mut self, task: &str, body: &str) -> Result<Comment, TaskError> {
        Ok(sqlx::query_as::<_, CommentRow>("INSERT INTO task_comments (id,owner_id,task_id,author_id,body) SELECT $3,owner_id,id,owner_id,$4 FROM tasks WHERE owner_id=$1 AND id=$2 RETURNING *")
            .bind(&self.owner).bind(task).bind(uuid::Uuid::new_v4().to_string()).bind(body).fetch_optional(&mut *self.tx).await?.ok_or(TaskError::NotFound)?.into())
    }
    pub async fn edit_comment(
        &mut self,
        task: &str,
        comment: &str,
        body: &str,
    ) -> Result<Comment, TaskError> {
        Ok(sqlx::query_as::<_, CommentRow>("UPDATE task_comments c SET body=$4,edited_at=GREATEST(clock_timestamp()::timestamptz(3),COALESCE(edited_at,created_at)+interval '1 millisecond') WHERE c.owner_id=$1 AND c.task_id=$2 AND c.id=$3 AND c.author_id=$1 AND EXISTS(SELECT 1 FROM tasks t WHERE t.owner_id=$1 AND t.id=$2) RETURNING c.*")
            .bind(&self.owner).bind(task).bind(comment).bind(body).fetch_optional(&mut *self.tx).await?.ok_or(TaskError::NotFound)?.into())
    }
    pub async fn delete_comment(&mut self, task: &str, comment: &str) -> Result<(), TaskError> {
        sqlx::query_scalar::<_, String>("DELETE FROM task_comments c WHERE c.owner_id=$1 AND c.task_id=$2 AND c.id=$3 AND c.author_id=$1 AND EXISTS(SELECT 1 FROM tasks t WHERE t.owner_id=$1 AND t.id=$2) RETURNING c.id")
            .bind(&self.owner).bind(task).bind(comment).fetch_optional(&mut *self.tx).await?.ok_or(TaskError::NotFound).map(|_| ())
    }
    pub async fn link(&mut self, task: &str, link: &str) -> Result<Link, TaskError> {
        Ok(sqlx::query_as::<_, LinkRow>("SELECT l.* FROM task_links l JOIN tasks t ON t.owner_id=l.owner_id AND t.id=l.task_id WHERE l.owner_id=$1 AND l.task_id=$2 AND l.id=$3")
            .bind(&self.owner).bind(task).bind(link).fetch_optional(&mut *self.tx).await?.ok_or(TaskError::NotFound)?.into())
    }
    pub async fn create_link(&mut self, task: &str, input: &LinkInput) -> Result<Link, TaskError> {
        Ok(sqlx::query_as::<_, LinkRow>("INSERT INTO task_links (id,owner_id,task_id,kind,label,url,note_id,sort_order) SELECT $3,owner_id,id,$4,$5,$6,$7,0 FROM tasks WHERE owner_id=$1 AND id=$2 RETURNING *")
            .bind(&self.owner).bind(task).bind(uuid::Uuid::new_v4().to_string()).bind(&input.kind).bind(&input.label).bind(&input.url).bind(&input.note_id)
            .fetch_optional(&mut *self.tx).await?.ok_or(TaskError::NotFound)?.into())
    }
    pub async fn edit_link(
        &mut self,
        task: &str,
        link: &str,
        input: &LinkInput,
    ) -> Result<(), TaskError> {
        sqlx::query_scalar::<_, String>(&format!("UPDATE task_links SET kind=$4,label=$5,url=$6,note_id=$7,updated_at={ADVANCE} WHERE owner_id=$1 AND task_id=$2 AND id=$3 RETURNING id"))
            .bind(&self.owner).bind(task).bind(link).bind(&input.kind).bind(&input.label).bind(&input.url).bind(&input.note_id)
            .fetch_optional(&mut *self.tx).await?.ok_or(TaskError::NotFound).map(|_| ())
    }
    pub async fn delete_link(&mut self, task: &str, link: &str) -> Result<(), TaskError> {
        sqlx::query_scalar::<_, String>("DELETE FROM task_links l WHERE l.owner_id=$1 AND l.task_id=$2 AND l.id=$3 AND EXISTS(SELECT 1 FROM tasks t WHERE t.owner_id=$1 AND t.id=$2) RETURNING l.id")
            .bind(&self.owner).bind(task).bind(link).fetch_optional(&mut *self.tx).await?.ok_or(TaskError::NotFound).map(|_| ())
    }
    pub async fn link_ids(&mut self, task: &str) -> Result<Vec<String>, TaskError> {
        Ok(sqlx::query_scalar(
            "SELECT id FROM task_links WHERE owner_id=$1 AND task_id=$2 ORDER BY sort_order,id",
        )
        .bind(&self.owner)
        .bind(task)
        .fetch_all(&mut *self.tx)
        .await?)
    }
    pub async fn link_ranks(&mut self, task: &str, ids: &[String]) -> Result<(), TaskError> {
        sqlx::query("UPDATE task_links l SET sort_order=r.rank-1,updated_at=GREATEST(clock_timestamp()::timestamptz(3),l.updated_at+interval '1 millisecond') FROM unnest($3::text[]) WITH ORDINALITY AS r(id,rank) WHERE l.owner_id=$1 AND l.task_id=$2 AND l.id=r.id AND l.sort_order<>r.rank-1")
            .bind(&self.owner).bind(task).bind(ids).execute(&mut *self.tx).await?;
        Ok(())
    }
}
fn scope<'a>(q: &mut QueryBuilder<'a, Postgres>, column: &'static str, scope: &'a Scope) {
    match scope {
        Scope::All => (),
        Scope::None => {
            q.push(" AND ").push(column).push(" IS NULL");
        }
        Scope::Id(id) => {
            q.push(" AND ").push(column).push("=").push_bind(id);
        }
    }
}

/// Called by the Space deletion transaction, after locking the same owner's user row.
/// Detach membership and normalize merged sibling lists in a single versioned update.
pub(crate) async fn detach_space_tasks(
    tx: &mut Transaction<'_, Postgres>,
    owner: &str,
    space: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("WITH ranks AS (SELECT id,row_number() OVER(PARTITION BY parent_id ORDER BY sort_order,id)-1 AS rank FROM tasks WHERE owner_id=$1 AND (space_id=$2 OR space_id IS NULL)) UPDATE tasks t SET space_id=NULL,sort_order=r.rank,version=t.version+1,updated_at=GREATEST(clock_timestamp()::timestamptz(3),t.updated_at+interval '1 millisecond') FROM ranks r WHERE t.owner_id=$1 AND t.id=r.id AND (t.space_id IS NOT NULL OR t.sort_order<>r.rank)")
        .bind(owner).bind(space).execute(&mut **tx).await?;
    Ok(())
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

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: String,
    space_id: Option<String>,
    parent_id: Option<String>,
    title: String,
    description: Option<String>,
    status: String,
    priority: Option<String>,
    flagged: bool,
    due_on: Option<NaiveDate>,
    due_at: Option<DateTime<Utc>>,
    due_timezone: Option<String>,
    completed_at: Option<DateTime<Utc>>,
    sort_order: i64,
    version: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    comment_count: i64,
    child_count: i64,
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
struct TreeRow {
    id: String,
    depth: i32,
}
#[derive(sqlx::FromRow)]
struct TagRow {
    id: String,
    name: String,
    created_at: DateTime<Utc>,
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
struct AssignedTagRow {
    task_id: String,
    #[sqlx(flatten)]
    tag: TagRow,
}
#[derive(sqlx::FromRow)]
struct CommentRow {
    id: String,
    task_id: String,
    author_id: String,
    body: String,
    created_at: DateTime<Utc>,
    edited_at: Option<DateTime<Utc>>,
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
struct LinkRow {
    id: String,
    task_id: String,
    kind: String,
    label: Option<String>,
    url: Option<String>,
    note_id: Option<String>,
    sort_order: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
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
