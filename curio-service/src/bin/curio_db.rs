use std::{env, process::ExitCode, time::Duration};

use curio_service::{
    config::validate_schema_name,
    database::{Database, DatabaseOptions},
};

#[tokio::main]
async fn main() -> ExitCode {
    let _ = dotenvy::dotenv();

    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("curio_db: {message}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), &'static str> {
    let mut arguments = env::args().skip(1);
    let command = arguments.next().ok_or("expected `migrate` or `verify`")?;
    if arguments.next().is_some() {
        return Err("unexpected command arguments");
    }

    let schema = required("CURIO_DB_SCHEMA")?;
    if !validate_schema_name(&schema) {
        return Err("CURIO_DB_SCHEMA is invalid");
    }

    let url_variable = match command.as_str() {
        "migrate" => "CURIO_MIGRATOR_DATABASE_URL",
        "verify" => "DATABASE_URL",
        _ => return Err("expected `migrate` or `verify`"),
    };
    let url = required(url_variable)?;
    let database = Database::connect(&DatabaseOptions {
        url,
        schema: schema.clone(),
        max_connections: 1,
        acquire_timeout: Duration::from_secs(10),
        application_name: format!("curio-db-{command}"),
    })
    .await
    .map_err(|_| "could not connect to the configured PostgreSQL schema")?;

    if command == "migrate" {
        verify_no_unknown_tables(&database).await?;
        database
            .migrate()
            .await
            .map_err(|_| "PostgreSQL migration failed")?;
    }

    database
        .verify_migrations()
        .await
        .map_err(|_| "the expected PostgreSQL migration is not applied")?;
    verify_catalog(&database).await?;
    database.close().await;

    println!("curio_db: {command} verified schema {schema}");
    Ok(())
}

async fn verify_catalog(database: &Database) -> Result<(), &'static str> {
    verify_no_unknown_tables(database).await?;

    let expected_tables = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM information_schema.tables
        WHERE table_schema = current_schema()
          AND table_type = 'BASE TABLE'
          AND table_name IN (
              'users',
              'conversations',
              'messages',
              'calendar_events',
              'sqlite_import_manifests'
          )
        "#,
    )
    .fetch_one(database.pool())
    .await
    .map_err(|_| "catalog verification failed")?;

    if expected_tables != 5 {
        return Err("one or more expected application tables are missing");
    }

    Ok(())
}

async fn verify_no_unknown_tables(database: &Database) -> Result<(), &'static str> {
    let unknown_tables = sqlx::query_scalar::<_, String>(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = current_schema()
          AND table_type = 'BASE TABLE'
          AND table_name NOT IN (
              '_sqlx_migrations',
              'users',
              'conversations',
              'messages',
              'calendar_events',
              'sqlite_import_manifests'
          )
        ORDER BY table_name
        "#,
    )
    .fetch_all(database.pool())
    .await
    .map_err(|_| "catalog verification failed")?;

    if !unknown_tables.is_empty() {
        return Err("the target schema contains unknown tables");
    }

    Ok(())
}

fn required(variable: &'static str) -> Result<String, &'static str> {
    env::var(variable)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or(variable)
}
