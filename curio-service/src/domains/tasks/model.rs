use crate::shared::serialization::serialize_timestamp;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

pub const STATUSES: [&str; 5] = ["todo", "in_progress", "blocked", "done", "cancelled"];
pub const PRIORITIES: [&str; 3] = ["low", "medium", "high"];

#[derive(Debug, PartialEq, Eq)]
pub enum TaskError {
    Invalid(&'static str),
    Query,
    NotFound,
    UnknownUser,
    Conflict,
    DuplicateTag,
    Internal,
}

/// Unlike Option<Option<T>>, explicit null is preserved by this deserializer.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum Field<T> {
    #[default]
    Missing,
    Null,
    Value(T),
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Field<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Option::<T>::deserialize(deserializer)?.map_or(Self::Null, Self::Value))
    }
}
impl<T> Field<T> {
    pub fn present(&self) -> bool {
        !matches!(self, Self::Missing)
    }
    pub fn nullable(self, current: Option<T>) -> Option<T> {
        match self {
            Self::Missing => current,
            Self::Null => None,
            Self::Value(value) => Some(value),
        }
    }
    pub fn required(self, current: T) -> Result<T, TaskError> {
        match self {
            Self::Missing => Ok(current),
            Self::Null => Err(TaskError::Invalid("This field cannot be null.")),
            Self::Value(value) => Ok(value),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub space_id: Field<String>,
    pub parent_id: Option<String>,
    #[serde(default = "todo")]
    pub status: String,
    pub priority: Option<String>,
    #[serde(default)]
    pub flagged: bool,
    pub due_on: Option<String>,
    pub due_at: Option<String>,
    pub due_timezone: Option<String>,
    #[serde(default)]
    pub tag_ids: Vec<String>,
    #[serde(default)]
    pub links: Vec<LinkInput>,
}
fn todo() -> String {
    "todo".into()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PatchTask {
    pub expected_version: i64,
    #[serde(default)]
    pub title: Field<String>,
    #[serde(default)]
    pub description: Field<String>,
    #[serde(default)]
    pub space_id: Field<String>,
    #[serde(default)]
    pub parent_id: Field<String>,
    #[serde(default)]
    pub status: Field<String>,
    #[serde(default)]
    pub priority: Field<String>,
    #[serde(default)]
    pub flagged: Field<bool>,
    #[serde(default)]
    pub due_on: Field<String>,
    #[serde(default)]
    pub due_at: Field<String>,
    #[serde(default)]
    pub due_timezone: Field<String>,
    #[serde(default)]
    pub sort_order: Field<i64>,
    #[serde(default)]
    pub tag_ids: Field<Vec<String>>,
}
impl PatchTask {
    pub fn validate(&self) -> Result<(), TaskError> {
        version(self.expected_version)?;
        if ![
            self.title.present(),
            self.description.present(),
            self.space_id.present(),
            self.parent_id.present(),
            self.status.present(),
            self.priority.present(),
            self.flagged.present(),
            self.due_on.present(),
            self.due_at.present(),
            self.due_timezone.present(),
            self.sort_order.present(),
            self.tag_ids.present(),
        ]
        .into_iter()
        .any(|p| p)
        {
            return Err(TaskError::Invalid("Patch must contain an editable field."));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub space_id: Option<String>,
    pub parent_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: Option<String>,
    pub flagged: bool,
    pub due_on: Option<NaiveDate>,
    #[serde(serialize_with = "optional_timestamp")]
    pub due_at: Option<DateTime<Utc>>,
    pub due_timezone: Option<String>,
    #[serde(serialize_with = "optional_timestamp")]
    pub completed_at: Option<DateTime<Utc>>,
    pub sort_order: i64,
    pub version: i64,
    #[serde(serialize_with = "serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<Tag>,
    pub links: Vec<Link>,
    pub comment_count: i64,
    pub child_count: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    #[serde(serialize_with = "serialize_timestamp")]
    pub created_at: DateTime<Utc>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagInput {
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    pub task_id: String,
    pub author_id: String,
    pub body: String,
    #[serde(serialize_with = "serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "optional_timestamp")]
    pub edited_at: Option<DateTime<Utc>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommentInput {
    pub body: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub id: String,
    pub task_id: String,
    pub kind: String,
    pub label: Option<String>,
    pub url: Option<String>,
    pub note_id: Option<String>,
    pub sort_order: i64,
    #[serde(serialize_with = "serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LinkInput {
    pub kind: String,
    pub label: Option<String>,
    pub url: Option<String>,
    pub note_id: Option<String>,
    pub sort_order: Option<i64>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PatchLink {
    #[serde(default)]
    pub kind: Field<String>,
    #[serde(default)]
    pub label: Field<String>,
    #[serde(default)]
    pub url: Field<String>,
    #[serde(default)]
    pub note_id: Field<String>,
    #[serde(default)]
    pub sort_order: Field<i64>,
}
impl PatchLink {
    pub fn merge(self, old: Link) -> Result<LinkInput, TaskError> {
        if ![
            self.kind.present(),
            self.label.present(),
            self.url.present(),
            self.note_id.present(),
            self.sort_order.present(),
        ]
        .into_iter()
        .any(|p| p)
        {
            return Err(TaskError::Invalid("Patch must contain an editable field."));
        }
        LinkInput {
            kind: self.kind.required(old.kind)?,
            label: self.label.nullable(old.label),
            url: self.url.nullable(old.url),
            note_id: self.note_id.nullable(old.note_id),
            sort_order: Some(self.sort_order.required(old.sort_order)?),
        }
        .validate()
    }
}
impl LinkInput {
    pub fn validate(mut self) -> Result<Self, TaskError> {
        self.label = optional_text(self.label, 200, false)?;
        if let Some(position) = self.sort_order {
            order(position)?;
        }
        match self.kind.as_str() {
            "web" if self.note_id.is_none() => {
                let url = text(
                    self.url
                        .as_deref()
                        .ok_or(TaskError::Invalid("Web links require a URL."))?,
                    2048,
                    false,
                )?;
                let parsed = reqwest::Url::parse(&url)
                    .map_err(|_| TaskError::Invalid("Invalid web URL."))?;
                // Validate the raw authority as well: URL parsers can discard empty userinfo.
                let authority = url
                    .split_once("://")
                    .map(|(_, rest)| rest.split(['/', '?', '#']).next().unwrap_or_default());
                if !matches!(parsed.scheme(), "http" | "https")
                    || parsed.host_str().is_none()
                    || !parsed.username().is_empty()
                    || parsed.password().is_some()
                    || authority.is_none_or(|value| value.contains('@'))
                {
                    return Err(TaskError::Invalid(
                        "Web URLs require HTTP(S), a host, and no credentials.",
                    ));
                }
                self.url = Some(url);
            }
            "curio_note" if self.url.is_none() => {
                self.note_id = Some(text(
                    self.note_id
                        .as_deref()
                        .ok_or(TaskError::Invalid("Note links require a note ID."))?,
                    240,
                    false,
                )?);
            }
            _ => {
                return Err(TaskError::Invalid(
                    "A link must contain only its kind's target.",
                ));
            }
        }
        Ok(self)
    }
}

pub fn optional_timestamp<S: Serializer>(
    value: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match value {
        Some(value) => serialize_timestamp(value, serializer),
        None => serializer.serialize_none(),
    }
}
pub fn text(value: &str, max: usize, multiline: bool) -> Result<String, TaskError> {
    if value
        .chars()
        .any(|c| c.is_control() && !(multiline && matches!(c, '\n' | '\r' | '\t')))
    {
        return Err(TaskError::Invalid(
            "Text contains unsupported control characters.",
        ));
    }
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max {
        return Err(TaskError::Invalid(
            "Text is empty or exceeds its character limit.",
        ));
    }
    Ok(value.to_owned())
}
pub fn optional_text(
    value: Option<String>,
    max: usize,
    multiline: bool,
) -> Result<Option<String>, TaskError> {
    value
        .map(|value| {
            if value.trim().is_empty() {
                if value
                    .chars()
                    .any(|c| c.is_control() && !(multiline && matches!(c, '\n' | '\r' | '\t')))
                {
                    return Err(TaskError::Invalid(
                        "Text contains unsupported control characters.",
                    ));
                }
                Ok(None)
            } else {
                text(&value, max, multiline).map(Some)
            }
        })
        .transpose()
        .map(Option::flatten)
}
pub fn id(value: &str) -> Result<String, TaskError> {
    Uuid::parse_str(value)
        .ok()
        .filter(|uuid| uuid.to_string() == value)
        .map(|_| value.to_owned())
        .ok_or(TaskError::Invalid("Expected a canonical UUID."))
}
pub fn ids(values: Vec<String>) -> Result<Vec<String>, TaskError> {
    if values.len() > 20 {
        return Err(TaskError::Invalid("At most 20 tags are allowed."));
    }
    let mut result = values
        .iter()
        .map(|value| id(value))
        .collect::<Result<Vec<_>, _>>()?;
    result.sort();
    if result.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(TaskError::Invalid("Duplicate tag IDs are not allowed."));
    }
    Ok(result)
}
pub fn date(value: &str) -> Result<NaiveDate, TaskError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()
        .filter(|d| (1..=9999).contains(&d.year()) && d.format("%Y-%m-%d").to_string() == value)
        .ok_or(TaskError::Invalid("Expected a YYYY-MM-DD calendar date."))
}
pub fn instant(value: &str) -> Result<DateTime<Utc>, TaskError> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|d| d.with_timezone(&Utc))
        .filter(|d| (1..=9999).contains(&d.year()) && d.timestamp_subsec_nanos() < 1_000_000_000)
        .and_then(|d| DateTime::from_timestamp_millis(d.timestamp_millis()))
        .ok_or(TaskError::Invalid("Expected an RFC 3339 instant."))
}
pub fn due(
    on: Option<NaiveDate>,
    at: Option<DateTime<Utc>>,
    timezone: Option<&str>,
) -> Result<(), TaskError> {
    if (on.is_some() && at.is_some()) || (at.is_some() != timezone.is_some()) {
        return Err(TaskError::Invalid(
            "Use dueOn or dueAt with dueTimezone; explicitly clear old due fields.",
        ));
    }
    if timezone.is_some_and(|zone| {
        zone.len() > 100
            || zone.is_empty()
            || !zone
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"/_+-".contains(&c))
            || zone.starts_with("posix/")
            || zone.starts_with("right/")
    }) {
        return Err(TaskError::Invalid("Invalid IANA timezone."));
    }
    Ok(())
}
pub fn status(value: &str) -> Result<(), TaskError> {
    if STATUSES.contains(&value) {
        Ok(())
    } else {
        Err(TaskError::Invalid("Invalid task status."))
    }
}
pub fn priority(value: Option<&str>) -> Result<(), TaskError> {
    if value.is_none_or(|v| PRIORITIES.contains(&v)) {
        Ok(())
    } else {
        Err(TaskError::Invalid("Invalid task priority."))
    }
}
pub fn version(value: i64) -> Result<(), TaskError> {
    if value > 0 {
        Ok(())
    } else {
        Err(TaskError::Invalid("expectedVersion must be positive."))
    }
}
pub fn order(value: i64) -> Result<usize, TaskError> {
    if (0..=i32::MAX as i64).contains(&value) {
        Ok(value as usize)
    } else {
        Err(TaskError::Invalid(
            "sortOrder must be a nonnegative list position.",
        ))
    }
}
pub fn tag_name(value: &str) -> Result<(String, String), TaskError> {
    let name = text(value, 80, false)?;
    let key = name.to_lowercase();
    Ok((name, key))
}
pub fn completion(
    old_status: &str,
    status: &str,
    completed: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Option<DateTime<Utc>> {
    if status == "done" {
        if old_status == "done" {
            completed.or(Some(now))
        } else {
            Some(now)
        }
    } else {
        None
    }
}

#[derive(Clone, Debug)]
pub struct TreeNode {
    pub id: String,
    pub depth: i32,
}
pub fn hierarchy(id: &str, ancestors: &[TreeNode], subtree_height: i32) -> Result<(), TaskError> {
    if ancestors.iter().any(|node| node.id == id) {
        return Err(TaskError::Invalid("A task cannot be its own ancestor."));
    }
    let parent_depth = ancestors.iter().map(|node| node.depth).max().unwrap_or(0);
    if parent_depth + subtree_height > 5 {
        return Err(TaskError::Invalid(
            "Task nesting is limited to five levels.",
        ));
    }
    Ok(())
}
