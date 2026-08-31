#![cfg(feature = "sqlite-import")]

use std::{env, path::Path, process::Command, time::Duration};

use curio_service::database::{Database, DatabaseOptions};
use sqlx::{
    PgPool,
    migrate::Migrator,
    postgres::PgPoolOptions,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

static SQLITE_MIGRATOR: Migrator = sqlx::migrate!("./migrations/sqlite");
const TARGET_URL_ENV: &str = "CURIO_SQLITE_IMPORT_TEST_TARGET_URL";

type TestResult<T = ()> = Result<T, &'static str>;
type FixtureMessage<'a> = (
    &'a str,
    &'a str,
    &'a str,
    &'a str,
    Option<&'a str>,
    Option<&'a str>,
);

#[tokio::test]
async fn nonempty_fixture_dry_run_import_and_second_run_refusal() {
    let Some(target_url) = env::var("CURIO_TEST_DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
    else {
        eprintln!("skipping SQLite importer integration test: CURIO_TEST_DATABASE_URL is not set");
        return;
    };

    if target_url.to_ascii_lowercase().contains("prod") {
        panic!("CURIO_TEST_DATABASE_URL must not identify a production target");
    }

    let admin = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(15))
        .connect(&target_url)
        .await
        .unwrap_or_else(|_| panic!("could not connect to the PostgreSQL test target"));
    verify_test_target(&admin)
        .await
        .unwrap_or_else(|message| panic!("{message}"));

    let schema = format!("curio_test_{}", uuid::Uuid::new_v4().simple());
    let create_schema = format!("CREATE SCHEMA \"{schema}\"");
    sqlx::query(&create_schema)
        .execute(&admin)
        .await
        .unwrap_or_else(|_| panic!("could not create a disposable PostgreSQL test schema"));

    let directory = tempfile::tempdir().expect("could not create a temporary fixture directory");
    let source = directory.path().join("legacy.sqlite");
    let scenario = run_scenario(&target_url, &schema, &source).await;

    let drop_schema = format!("DROP SCHEMA \"{schema}\" CASCADE");
    let cleanup = sqlx::query(&drop_schema).execute(&admin).await;
    admin.close().await;

    if cleanup.is_err() {
        panic!("could not drop the disposable PostgreSQL test schema");
    }
    if let Err(message) = scenario {
        panic!("{message}");
    }
}

async fn run_scenario(target_url: &str, schema: &str, source: &Path) -> TestResult {
    create_nonempty_fixture(source).await?;

    let dry_run = run_importer(target_url, schema, source, false)?;
    if !dry_run.status.success() {
        return Err("the importer dry run rejected a valid non-empty fixture");
    }

    let apply = run_importer(target_url, schema, source, true)?;
    if !apply.status.success() {
        return Err("the importer could not apply a valid non-empty fixture");
    }

    verify_imported_fixture(target_url, schema).await?;

    let second_apply = run_importer(target_url, schema, source, true)?;
    if second_apply.status.success() {
        return Err("a second SQLite import unexpectedly succeeded");
    }
    let second_error = String::from_utf8_lossy(&second_apply.stderr);
    if !second_error.contains("already has an SQLite import manifest") {
        return Err("the second import did not fail because of the manifest guard");
    }

    Ok(())
}

async fn verify_test_target(pool: &PgPool) -> TestResult {
    let (database, user) =
        sqlx::query_as::<_, (String, String)>("SELECT current_database(), current_user")
            .fetch_one(pool)
            .await
            .map_err(|_| "the PostgreSQL test target identity could not be read")?;

    if database.to_ascii_lowercase().contains("prod") || user.to_ascii_lowercase().contains("prod")
    {
        return Err("the PostgreSQL test database or role appears to be a production target");
    }
    Ok(())
}

async fn create_nonempty_fixture(path: &Path) -> TestResult {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|_| "the generated SQLite fixture could not be opened")?;

    SQLITE_MIGRATOR
        .run(&pool)
        .await
        .map_err(|_| "the archived SQLite migrations could not create the fixture")?;

    sqlx::query(
        r#"
        INSERT INTO users (
            id, name, email, avatar_url, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )
    .bind("user-a")
    .bind("Curio User")
    .bind("curio@example.com")
    .bind(Option::<&str>::None)
    .bind("2026-08-31 12:00:00")
    .bind("2026-08-31T07:30:00-05:00")
    .execute(&pool)
    .await
    .map_err(|_| "the fixture user could not be inserted")?;

    sqlx::query(
        r#"
        INSERT INTO conversations (id, created_at, updated_at)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind("conversation-1")
    .bind("2026-08-31 12:00:00")
    .bind("2026-08-31 12:00:01")
    .execute(&pool)
    .await
    .map_err(|_| "the fixture conversation could not be inserted")?;

    let messages: [FixtureMessage<'_>; 5] = [
        (
            "message-user",
            "user",
            "hello",
            "completed",
            None::<&str>,
            None::<&str>,
        ),
        (
            "message-completed",
            "assistant",
            "hello back",
            "completed",
            Some("response-1"),
            None,
        ),
        ("message-pending", "assistant", "", "pending", None, None),
        (
            "message-failed",
            "assistant",
            "partial response",
            "failed",
            None,
            Some("provider_error"),
        ),
        (
            "message-interrupted",
            "assistant",
            "partial disconnect",
            "interrupted",
            None,
            Some("client_disconnected"),
        ),
    ];
    for (id, role, content, status, response_id, error_code) in messages {
        sqlx::query(
            r#"
            INSERT INTO messages (
                id, conversation_id, role, content, status, response_id,
                error_code, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
        )
        .bind(id)
        .bind("conversation-1")
        .bind(role)
        .bind(content)
        .bind(status)
        .bind(response_id)
        .bind(error_code)
        .bind("2026-08-31 12:00:00")
        .bind("2026-08-31 12:00:01")
        .execute(&pool)
        .await
        .map_err(|_| "a fixture message could not be inserted")?;
    }

    sqlx::query(
        r#"
        INSERT INTO calendar_events (
            id, user_id, title, description, status, priority, all_day,
            start_date, end_date, starts_at, ends_at, created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7,
            ?8, ?9, ?10, ?11, ?12, ?13
        )
        "#,
    )
    .bind("event-1")
    .bind("user-a")
    .bind("Offsite")
    .bind(Some("Planning day"))
    .bind(Some("scheduled"))
    .bind(Some("medium"))
    .bind(1_i64)
    .bind("2026-09-01")
    .bind(Option::<&str>::None)
    .bind("2026-09-01T00:00:00.000Z")
    .bind("2026-09-02T00:00:00.000Z")
    .bind("2026-08-31 12:00:00")
    .bind("2026-08-31 12:00:01")
    .execute(&pool)
    .await
    .map_err(|_| "the fixture calendar event could not be inserted")?;

    sqlx::query(
        r#"
        INSERT INTO calendar_events (
            id, user_id, title, description, status, priority, all_day,
            start_date, end_date, starts_at, ends_at, created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7,
            ?8, ?9, ?10, ?11, ?12, ?13
        )
        "#,
    )
    .bind("event-2")
    .bind("user-a")
    .bind("Timed review")
    .bind(Option::<&str>::None)
    .bind(Option::<&str>::None)
    .bind(Option::<&str>::None)
    .bind(0_i64)
    .bind("2026-09-01T09:00:00-05:00")
    .bind(Some("2026-09-01T10:00:00-05:00"))
    .bind("2026-09-01T14:00:00.000Z")
    .bind("2026-09-01T15:00:00.000Z")
    .bind("2026-08-31 12:00:00")
    .bind("2026-08-31 12:00:01")
    .execute(&pool)
    .await
    .map_err(|_| "the fixture timed calendar event could not be inserted")?;

    pool.close().await;
    Ok(())
}

fn run_importer(
    target_url: &str,
    schema: &str,
    source: &Path,
    apply: bool,
) -> TestResult<std::process::Output> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_import_sqlite"));
    command
        .arg("--source")
        .arg(source)
        .arg("--target-url-env")
        .arg(TARGET_URL_ENV)
        .arg("--target-schema")
        .arg(schema)
        .env(TARGET_URL_ENV, target_url);
    if apply {
        command.arg("--apply");
    }

    command
        .output()
        .map_err(|_| "the SQLite importer process could not be started")
}

async fn verify_imported_fixture(target_url: &str, schema: &str) -> TestResult {
    let database = Database::connect(&DatabaseOptions {
        url: target_url.to_owned(),
        schema: schema.to_owned(),
        max_connections: 1,
        acquire_timeout: Duration::from_secs(15),
        application_name: "curio-sqlite-import-test".to_owned(),
    })
    .await
    .map_err(|_| "the imported fixture could not be reopened")?;

    let counts = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
        r#"
        SELECT
            (SELECT COUNT(*)::bigint FROM users),
            (SELECT COUNT(*)::bigint FROM conversations),
            (SELECT COUNT(*)::bigint FROM messages),
            (SELECT COUNT(*)::bigint FROM calendar_events),
            (SELECT COUNT(*)::bigint FROM sqlite_import_manifests)
        "#,
    )
    .fetch_one(database.pool())
    .await
    .map_err(|_| "the imported fixture counts could not be read")?;
    if counts != (1, 1, 5, 2, 1) {
        database.close().await;
        return Err("the imported fixture counts were incorrect");
    }

    let message_order = sqlx::query_as::<_, (i64, String)>(
        "SELECT sort_order, role FROM messages ORDER BY created_at, sort_order",
    )
    .fetch_all(database.pool())
    .await
    .map_err(|_| "the imported message order could not be read")?;
    database.close().await;

    if message_order
        != [
            (1, "user".to_owned()),
            (2, "assistant".to_owned()),
            (3, "assistant".to_owned()),
            (4, "assistant".to_owned()),
            (5, "assistant".to_owned()),
        ]
    {
        return Err("SQLite rowid ordering was not preserved in PostgreSQL");
    }
    Ok(())
}
