//! Fallible SQLx encoding and placeholder allocation, shared by all builders.
use sqlx::{Arguments, Encode, Postgres, QueryBuilder, Type, postgres::PgArguments};

/// SQLx owns the encoder and placeholder numbering. Keep the first encoding
/// error until build so fluent composition never panics on a bad value.
pub(super) struct BoundSql {
    pub(super) sql: String,
    pub(super) arguments: PgArguments,
    pub(super) error: Option<sqlx::Error>,
}
impl BoundSql {
    pub(super) fn new(sql: &str) -> Self {
        Self {
            sql: sql.to_owned(),
            arguments: PgArguments::default(),
            error: None,
        }
    }
    pub(super) fn push(&mut self, sql: &str) -> &mut Self {
        self.sql.push_str(sql);
        self
    }
    pub(super) fn push_bind<'args, T>(&mut self, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        if self.error.is_some() {
            return self;
        }
        if self.arguments.len() >= 65_535 {
            self.error = Some(sqlx::Error::Protocol(
                "PostgreSQL bind parameter limit exceeded".to_owned(),
            ));
            return self;
        }
        if let Err(error) = self.arguments.add(value) {
            self.error = Some(sqlx::Error::Encode(error));
        } else if self.arguments.format_placeholder(&mut self.sql).is_err() {
            self.error = Some(sqlx::Error::Protocol(
                "could not format a bind placeholder".to_owned(),
            ));
        }
        self
    }
    pub(super) fn build(self) -> Result<QueryBuilder<'static, Postgres>, sqlx::Error> {
        match self.error {
            Some(error) => Err(error),
            None => Ok(QueryBuilder::with_arguments(self.sql, self.arguments)),
        }
    }
}
