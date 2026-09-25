//! Profile data and validation; no HTTP or SQL dependencies.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserRecord {
    pub id: String,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    #[serde(serialize_with = "crate::shared::serialization::serialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "crate::shared::serialization::serialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveUserRequest {
    pub id: String,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserError {
    Invalid,
    EmailConflict,
    Unavailable,
}

/// Validated profile values accepted by persistence.
pub struct UserProfile<'a> {
    id: &'a str,
    name: &'a str,
    email: &'a str,
    avatar_url: Option<&'a str>,
}

impl<'a> UserProfile<'a> {
    pub fn parse(input: &'a SaveUserRequest) -> Result<Self, UserError> {
        let profile = Self {
            id: input.id.trim(),
            name: input.name.trim(),
            email: input.email.trim(),
            avatar_url: input
                .avatar_url
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        };
        if profile.id.is_empty() || profile.name.is_empty() || profile.email.is_empty() {
            return Err(UserError::Invalid);
        }
        Ok(profile)
    }
    pub fn id(&self) -> &str {
        self.id
    }
    pub fn name(&self) -> &str {
        self.name
    }
    pub fn email(&self) -> &str {
        self.email
    }
    pub fn avatar_url(&self) -> Option<&str> {
        self.avatar_url
    }
}
