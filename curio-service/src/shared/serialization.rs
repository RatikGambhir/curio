//! Shared wire timestamp formatting, independent of database lifecycle.
use chrono::{DateTime, Utc};
use serde::Serializer;

pub(crate) fn serialize_timestamp<S>(
    value: &DateTime<Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
}
