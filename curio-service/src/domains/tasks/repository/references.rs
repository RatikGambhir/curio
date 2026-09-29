//! Owned Space lookup and PostgreSQL timezone validation.
use super::super::model::TaskError;
use super::{TaskTransaction, columns::TimezoneColumn};
use crate::{
    adapters::postgres::query::{Select, col, int, val},
    domains::spaces::repository::SpaceColumn,
};

impl TaskTransaction {
    pub async fn space(&mut self, id: &str) -> Result<(), TaskError> {
        Select::from("spaces")
            .columns([SpaceColumn::Id])
            .where_(col(SpaceColumn::OwnerId).eq(val(&self.owner)))
            .where_(col(SpaceColumn::Id).eq(val(id)))
            .build()?
            .build_query_scalar::<String>()
            .fetch_optional(&mut *self.tx)
            .await?
            .ok_or(TaskError::NotFound)
            .map(|_| ())
    }
    pub async fn timezone(&mut self, zone: &str) -> Result<(), TaskError> {
        let known = Select::from("pg_catalog.pg_timezone_names")
            .columns([int(1)])
            .where_(col(TimezoneColumn::Name).eq(val(zone)));
        let valid: bool = Select::expressions([known.exists()])
            .build()?
            .build_query_scalar()
            .fetch_one(&mut *self.tx)
            .await?;
        if valid {
            Ok(())
        } else {
            Err(TaskError::Invalid("Unknown IANA timezone."))
        }
    }
}
