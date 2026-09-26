//! Canonical filters and bounded cursors; no SQL or HTTP dependencies.
use super::model::{self, Task, TaskError};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListQuery {
    pub space_id: Option<String>,
    pub parent_id: Option<String>,
    pub statuses: Option<String>,
    pub priority: Option<String>,
    pub flagged: Option<bool>,
    pub tag_ids: Option<String>,
    pub tag_mode: Option<String>,
    pub has_due_date: Option<bool>,
    pub due_kind: Option<String>,
    pub due_from: Option<String>,
    pub due_before: Option<String>,
    pub completed_from: Option<String>,
    pub completed_before: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Scope {
    #[default]
    All,
    None,
    Id(String),
}
impl Scope {
    fn parse(value: Option<String>) -> Result<Self, TaskError> {
        match value.as_deref() {
            None => Ok(Self::All),
            Some("none") => Ok(Self::None),
            Some(value) => model::id(value).map(Self::Id),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sort {
    Created,
    DueDate,
    DueTime,
    Priority,
    Manual,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Filters {
    pub space: Scope,
    pub parent: Scope,
    pub statuses: Vec<String>,
    pub priority: Option<String>,
    pub flagged: Option<bool>,
    pub tags: Vec<String>,
    pub all_tags: bool,
    pub has_due: Option<bool>,
    pub due_kind: Option<String>,
    pub due_from: Option<String>,
    pub due_before: Option<String>,
    pub completed_from: Option<DateTime<Utc>>,
    pub completed_before: Option<DateTime<Utc>>,
    pub sort: Sort,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TaskCursor {
    pub filters: Filters,
    pub key: i64,
    pub id: String,
}
pub struct TaskPage {
    pub filters: Filters,
    pub limit: usize,
    pub cursor: Option<TaskCursor>,
}

impl TaskPage {
    pub fn parse(query: ListQuery) -> Result<Self, TaskError> {
        Self::parse_inner(query).map_err(|_| TaskError::Query)
    }
    fn parse_inner(query: ListQuery) -> Result<Self, TaskError> {
        let limit = limit(query.limit)?;
        let mut statuses = match query.statuses.as_deref() {
            None => vec!["todo".into(), "in_progress".into(), "blocked".into()],
            Some("all") => model::STATUSES.iter().map(|s| (*s).into()).collect(),
            Some(value) => csv(value, 5)?,
        };
        for status in &statuses {
            model::status(status)?;
        }
        statuses.sort();
        model::priority(query.priority.as_deref())?;
        let tags = query
            .tag_ids
            .as_deref()
            .map(|v| csv(v, 20).and_then(model::ids))
            .transpose()?
            .unwrap_or_default();
        let all_tags = match query.tag_mode.as_deref() {
            None | Some("all") => true,
            Some("any") => false,
            _ => return Err(TaskError::Query),
        };
        if query.tag_mode.is_some() && tags.is_empty() {
            return Err(TaskError::Query);
        }
        let space = Scope::parse(query.space_id)?;
        let parent = Scope::parse(query.parent_id)?;
        if query
            .due_kind
            .as_deref()
            .is_some_and(|k| k != "date" && k != "timed")
        {
            return Err(TaskError::Query);
        }
        if query.has_due_date == Some(false)
            && (query.due_kind.is_some() || query.due_from.is_some() || query.due_before.is_some())
        {
            return Err(TaskError::Query);
        }
        let parse_due = |value: String| -> Result<String, TaskError> {
            match query.due_kind.as_deref() {
                Some("date") => model::date(&value).map(|date| date.to_string()),
                Some("timed") => model::instant(&value)
                    .map(|date| date.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
                _ => Err(TaskError::Query),
            }
        };
        let due_from = query.due_from.map(&parse_due).transpose()?;
        let due_before = query.due_before.map(parse_due).transpose()?;
        if due_from
            .as_ref()
            .zip(due_before.as_ref())
            .is_some_and(|(a, b)| a >= b)
        {
            return Err(TaskError::Query);
        }
        let completed_from = query
            .completed_from
            .as_deref()
            .map(model::instant)
            .transpose()?;
        let completed_before = query
            .completed_before
            .as_deref()
            .map(model::instant)
            .transpose()?;
        if completed_from
            .zip(completed_before)
            .is_some_and(|(a, b)| a >= b)
        {
            return Err(TaskError::Query);
        }
        let sort = match query.sort.as_deref().unwrap_or("created") {
            "created" => Sort::Created,
            "due" => match query.due_kind.as_deref() {
                Some("date") => Sort::DueDate,
                Some("timed") => Sort::DueTime,
                _ => return Err(TaskError::Query),
            },
            "priority" => Sort::Priority,
            "manual" if space != Scope::All || parent != Scope::All => Sort::Manual,
            _ => return Err(TaskError::Query),
        };
        let filters = Filters {
            space,
            parent,
            statuses,
            priority: query.priority,
            flagged: query.flagged,
            tags,
            all_tags,
            has_due: query.has_due_date,
            due_kind: query.due_kind,
            due_from,
            due_before,
            completed_from,
            completed_before,
            sort,
        };
        let cursor: Option<TaskCursor> = query.cursor.as_deref().map(decode).transpose()?;
        if let Some(cursor) = &cursor {
            model::id(&cursor.id)?;
            if cursor.filters != filters {
                return Err(TaskError::Query);
            }
            match sort {
                Sort::Created | Sort::DueTime => {
                    cursor_time(cursor.key)?;
                }
                Sort::DueDate => {
                    cursor_date(cursor.key)?;
                }
                Sort::Priority if !(0..=3).contains(&cursor.key) => return Err(TaskError::Query),
                Sort::Manual if cursor.key < 0 => return Err(TaskError::Query),
                _ => (),
            }
        }
        Ok(Self {
            filters,
            limit,
            cursor,
        })
    }
    pub fn next(&self, task: &Task) -> Result<String, TaskError> {
        let key = match self.filters.sort {
            Sort::Created => task.created_at.timestamp_millis(),
            Sort::DueDate => task.due_on.ok_or(TaskError::Internal)?.num_days_from_ce() as i64,
            Sort::DueTime => task.due_at.ok_or(TaskError::Internal)?.timestamp_millis(),
            Sort::Priority => match task.priority.as_deref() {
                Some("high") => 0,
                Some("medium") => 1,
                Some("low") => 2,
                _ => 3,
            },
            Sort::Manual => task.sort_order,
        };
        encode(&TaskCursor {
            filters: self.filters.clone(),
            key,
            id: task.id.clone(),
        })
    }
}

fn csv(value: &str, max: usize) -> Result<Vec<String>, TaskError> {
    let values: Vec<_> = value.split(',').map(str::to_owned).collect();
    if values.len() > max || values.iter().any(String::is_empty) {
        return Err(TaskError::Query);
    }
    let mut unique = values.clone();
    unique.sort();
    unique.dedup();
    if values.len() != unique.len() {
        return Err(TaskError::Query);
    }
    Ok(values)
}
pub fn cursor_time(value: i64) -> Result<DateTime<Utc>, TaskError> {
    DateTime::from_timestamp_millis(value)
        .filter(|d| (1..=9999).contains(&d.year()))
        .ok_or(TaskError::Query)
}
pub fn cursor_date(value: i64) -> Result<NaiveDate, TaskError> {
    i32::try_from(value)
        .ok()
        .and_then(NaiveDate::from_num_days_from_ce_opt)
        .filter(|d| (1..=9999).contains(&d.year()))
        .ok_or(TaskError::Query)
}
fn limit(value: Option<u32>) -> Result<usize, TaskError> {
    let value = value.unwrap_or(50);
    if (1..=100).contains(&value) {
        Ok(value as usize)
    } else {
        Err(TaskError::Query)
    }
}
pub fn encode<T: Serialize>(value: &T) -> Result<String, TaskError> {
    serde_json::to_vec(value)
        .map(|bytes| URL_SAFE_NO_PAD.encode(bytes))
        .map_err(|_| TaskError::Internal)
}
fn decode<T: serde::de::DeserializeOwned>(value: &str) -> Result<T, TaskError> {
    if value.is_empty() || value.len() > 8192 {
        return Err(TaskError::Query);
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| TaskError::Query)?;
    serde_json::from_slice(&bytes).map_err(|_| TaskError::Query)
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PageQuery {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreatedCursor {
    pub scope: String,
    pub created: i64,
    pub id: String,
}
pub struct CreatedPage {
    pub limit: usize,
    pub scope: String,
    pub cursor: Option<CreatedCursor>,
}
impl CreatedPage {
    pub fn parse(query: PageQuery, scope: String) -> Result<Self, TaskError> {
        let cursor: Option<CreatedCursor> = query.cursor.as_deref().map(decode).transpose()?;
        if let Some(cursor) = &cursor {
            if cursor.scope != scope {
                return Err(TaskError::Query);
            }
            model::id(&cursor.id).map_err(|_| TaskError::Query)?;
            cursor_time(cursor.created)?;
        }
        Ok(Self {
            limit: limit(query.limit)?,
            scope,
            cursor,
        })
    }
    pub fn next(&self, created: DateTime<Utc>, id: &str) -> Result<String, TaskError> {
        encode(&CreatedCursor {
            scope: self.scope.clone(),
            created: created.timestamp_millis(),
            id: id.to_owned(),
        })
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeleteQuery {
    pub expected_version: i64,
}
