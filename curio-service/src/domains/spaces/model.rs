use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::serialization::serialize_timestamp;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Space {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(serialize_with = "serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateSpaceInput {
    pub name: String,
    pub description: Option<String>,
}

pub struct NewSpace<'a> {
    id: String,
    name: &'a str,
    description: Option<&'a str>,
}

impl<'a> NewSpace<'a> {
    pub fn parse(input: &'a CreateSpaceInput) -> Result<Self, SpaceError> {
        if input.name.chars().any(char::is_control) {
            return Err(SpaceError::Invalid(
                "Name must not contain control characters.",
            ));
        }
        let name = input.name.trim();
        if name.is_empty() || name.chars().count() > 120 {
            return Err(SpaceError::Invalid(
                "Name must contain 1 to 120 characters.",
            ));
        }
        if input
            .description
            .as_deref()
            .is_some_and(|value| value.chars().any(char::is_control))
        {
            return Err(SpaceError::Invalid(
                "Description must not contain control characters.",
            ));
        }
        let description = input.description.as_deref().map(str::trim);
        if description.is_some_and(|value| value.chars().count() > 2000) {
            return Err(SpaceError::Invalid(
                "Description must contain at most 2000 characters.",
            ));
        }
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn name(&self) -> &str {
        self.name
    }
    pub fn description(&self) -> Option<&str> {
        self.description
    }
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListSpacesQuery {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

pub struct SpacePage {
    limit: usize,
    cursor: Option<SpaceCursor>,
}

impl SpacePage {
    pub fn parse(query: ListSpacesQuery) -> Result<Self, SpaceError> {
        let limit = query.limit.unwrap_or(50);
        if !(1..=100).contains(&limit) {
            return Err(SpaceError::InvalidQuery);
        }
        let cursor = query
            .cursor
            .as_deref()
            .map(SpaceCursor::decode)
            .transpose()?;
        Ok(Self {
            limit: limit as usize,
            cursor,
        })
    }

    pub fn limit(&self) -> usize {
        self.limit
    }
    pub fn cursor(&self) -> Option<&SpaceCursor> {
        self.cursor.as_ref()
    }
}

pub struct SpaceCursor {
    created_at: DateTime<Utc>,
    id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CursorPayload {
    created_at: i64,
    id: String,
}

impl SpaceCursor {
    pub fn encode(space: &Space) -> Result<String, SpaceError> {
        let bytes = serde_json::to_vec(&CursorPayload {
            created_at: space.created_at.timestamp_millis(),
            id: space.id.clone(),
        })
        .map_err(|_| SpaceError::Internal)?;
        Ok(URL_SAFE_NO_PAD.encode(bytes))
    }

    fn decode(cursor: &str) -> Result<Self, SpaceError> {
        // Bound work on untrusted cursors before decoding or parsing.
        if cursor.is_empty() || cursor.len() > 256 {
            return Err(SpaceError::InvalidQuery);
        }
        let bytes = URL_SAFE_NO_PAD
            .decode(cursor)
            .map_err(|_| SpaceError::InvalidQuery)?;
        let payload: CursorPayload =
            serde_json::from_slice(&bytes).map_err(|_| SpaceError::InvalidQuery)?;
        let created_at = DateTime::from_timestamp_millis(payload.created_at)
            .filter(|value| (1..=9999).contains(&value.year()))
            .ok_or(SpaceError::InvalidQuery)?;
        let id = Uuid::parse_str(&payload.id).map_err(|_| SpaceError::InvalidQuery)?;
        if id.to_string() != payload.id {
            return Err(SpaceError::InvalidQuery);
        }
        Ok(Self {
            created_at,
            id: payload.id,
        })
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpacesResponse {
    pub spaces: Vec<Space>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SpaceError {
    Invalid(&'static str),
    InvalidQuery,
    DuplicateName,
    UnknownUser,
    NotFound,
    Internal,
}
