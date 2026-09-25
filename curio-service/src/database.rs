use std::{fmt, io, str::FromStr, time::Duration};

use sqlx::{
    PgPool,
    migrate::{MigrateError, Migrator},
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::config::validate_schema_name;

pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations/postgres");

#[derive(Clone, PartialEq, Eq)]
pub struct DatabaseOptions {
    pub url: String,
    pub schema: String,
    pub max_connections: u32,
    pub acquire_timeout: Duration,
    pub application_name: String,
}

impl fmt::Debug for DatabaseOptions {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DatabaseOptions")
            .field("url", &"[REDACTED]")
            .field("schema", &self.schema)
            .field("max_connections", &self.max_connections)
            .field("acquire_timeout", &self.acquire_timeout)
            .field("application_name", &self.application_name)
            .finish()
    }
}

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
    schema: String,
}

impl Database {
    /// Opens and validates a PostgreSQL pool without applying any DDL.
    pub async fn connect(options: &DatabaseOptions) -> Result<Self, sqlx::Error> {
        if !validate_schema_name(&options.schema) {
            return Err(configuration_error(
                "CURIO_DB_SCHEMA is not a safe identifier",
            ));
        }

        let connect_options =
            PgConnectOptions::from_str(&options.url)?.application_name(&options.application_name);
        let schema = options.schema.clone();
        let search_path = format!("{schema},pg_catalog");

        let pool = PgPoolOptions::new()
            .max_connections(options.max_connections)
            .acquire_timeout(options.acquire_timeout)
            .after_connect(move |connection, _metadata| {
                let schema = schema.clone();
                let search_path = search_path.clone();
                Box::pin(async move {
                    sqlx::query("SET TIME ZONE 'UTC'")
                        .execute(&mut *connection)
                        .await?;
                    sqlx::query_scalar::<_, String>("SELECT set_config('search_path', $1, false)")
                        .bind(search_path)
                        .fetch_one(&mut *connection)
                        .await?;

                    let current_schema =
                        sqlx::query_scalar::<_, Option<String>>("SELECT current_schema()")
                            .fetch_one(&mut *connection)
                            .await?;
                    if current_schema.as_deref() != Some(schema.as_str()) {
                        return Err(configuration_error(
                            "configured database schema does not exist or is not accessible",
                        ));
                    }

                    Ok(())
                })
            })
            .connect_with(connect_options)
            .await?;

        Ok(Self {
            pool,
            schema: options.schema.clone(),
        })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub async fn migrate(&self) -> Result<(), MigrateError> {
        MIGRATOR.run(&self.pool).await
    }

    pub async fn verify_migrations(&self) -> Result<(), sqlx::Error> {
        let Some(expected_version) = MIGRATOR.iter().last().map(|migration| migration.version)
        else {
            return Err(configuration_error("no PostgreSQL migrations are embedded"));
        };

        let migration_is_current = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM _sqlx_migrations
                WHERE version = $1 AND success = TRUE
            )
            "#,
        )
        .bind(expected_version)
        .fetch_one(&self.pool)
        .await?;

        if !migration_is_current {
            return Err(configuration_error(
                "database schema has not applied the expected migration",
            ));
        }

        Ok(())
    }

    pub async fn readiness(&self) -> Result<(), sqlx::Error> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        self.verify_migrations().await
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}

fn configuration_error(message: &'static str) -> sqlx::Error {
    sqlx::Error::Configuration(Box::new(io::Error::new(
        io::ErrorKind::InvalidInput,
        message,
    )))
}

#[cfg(test)]
#[path = "../tests/unit/database.rs"]
mod tests;
