use std::{env, time::Duration};

use curio_service::adapters::postgres::client::{Database, DatabaseOptions, validate_schema_name};
use sqlx::{PgPool, postgres::PgPoolOptions};

const TEST_DATABASE_ENV: &str = "CURIO_TEST_DATABASE_URL";

pub struct PostgresFixture {
    database: Database,
    admin_pool: PgPool,
    database_url: String,
    schema: String,
}

impl PostgresFixture {
    pub async fn provision(test_name: &str) -> Option<Self> {
        let Some(database_url) = env::var(TEST_DATABASE_ENV)
            .ok()
            .filter(|value| !value.trim().is_empty())
        else {
            eprintln!(
                "skipping PostgreSQL-backed test {test_name}: {TEST_DATABASE_ENV} is not set"
            );
            return None;
        };

        assert!(
            !database_url.to_ascii_lowercase().contains("prod"),
            "{TEST_DATABASE_ENV} must not contain 'prod'"
        );

        let admin_pool = PgPoolOptions::new()
            .max_connections(2)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&database_url)
            .await
            .unwrap_or_else(|error| {
                panic!("failed to connect to {TEST_DATABASE_ENV} for {test_name}: {error}")
            });

        let (current_user, current_database): (String, String) =
            sqlx::query_as("SELECT current_user::text, current_database()::text")
                .fetch_one(&admin_pool)
                .await
                .unwrap_or_else(|error| {
                    panic!("failed to inspect the PostgreSQL test target for {test_name}: {error}")
                });
        assert!(
            !current_user.to_ascii_lowercase().contains("prod"),
            "PostgreSQL test user must not contain 'prod'"
        );
        assert!(
            !current_database.to_ascii_lowercase().contains("prod"),
            "PostgreSQL test database must not contain 'prod'"
        );

        let schema = format!("curio_test_{}", uuid::Uuid::new_v4().simple());
        assert!(
            validate_schema_name(&schema),
            "generated test schema must be a safe identifier"
        );

        sqlx::query(&format!("CREATE SCHEMA {schema}"))
            .execute(&admin_pool)
            .await
            .unwrap_or_else(|error| {
                panic!("failed to create disposable schema for {test_name}: {error}")
            });

        let options = database_options(&database_url, &schema);
        let database = match Database::connect(&options).await {
            Ok(database) => database,
            Err(error) => {
                drop_schema(&admin_pool, &schema).await;
                panic!("failed to connect to disposable schema for {test_name}: {error}");
            }
        };

        if let Err(error) = database.migrate().await {
            database.close().await;
            drop_schema(&admin_pool, &schema).await;
            panic!("failed to migrate disposable schema for {test_name}: {error}");
        }

        database
            .readiness()
            .await
            .unwrap_or_else(|error| panic!("migrated test schema is not ready: {error}"));

        Some(Self {
            database,
            admin_pool,
            database_url,
            schema,
        })
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    #[allow(dead_code)] // Used by integration tests; the file is also included by unit tests.
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    #[allow(dead_code)] // Used by integration tests; the file is also included by unit tests.
    pub fn schema(&self) -> &str {
        &self.schema
    }

    #[allow(dead_code)] // Used by unit tests; the file is also compiled by integration tests.
    pub async fn reconnect(&self) -> Database {
        Database::connect(&database_options(&self.database_url, &self.schema))
            .await
            .expect("failed to reconnect to disposable PostgreSQL schema")
    }

    pub async fn cleanup(self) {
        self.database.close().await;
        drop_schema(&self.admin_pool, &self.schema).await;
        self.admin_pool.close().await;
    }
}

fn database_options(database_url: &str, schema: &str) -> DatabaseOptions {
    DatabaseOptions {
        url: database_url.to_owned(),
        schema: schema.to_owned(),
        max_connections: 5,
        acquire_timeout: Duration::from_secs(5),
        application_name: "curio-service-test".to_owned(),
    }
}

async fn drop_schema(admin_pool: &PgPool, schema: &str) {
    assert!(
        validate_schema_name(schema),
        "cleanup schema must be a safe identifier"
    );
    assert!(
        schema.starts_with("curio_test_"),
        "refusing to drop a non-test schema"
    );
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(admin_pool)
        .await
        .expect("failed to drop disposable PostgreSQL schema");
}
