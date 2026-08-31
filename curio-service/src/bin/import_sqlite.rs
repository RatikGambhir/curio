#[cfg(not(feature = "sqlite-import"))]
fn main() {
    eprintln!("the import_sqlite binary requires the sqlite-import feature");
    std::process::exit(2);
}

#[cfg(feature = "sqlite-import")]
#[tokio::main]
async fn main() {
    if let Err(error) = importer::run().await {
        eprintln!("SQLite import failed: {error}");
        std::process::exit(1);
    }
}

#[cfg(feature = "sqlite-import")]
mod importer {
    use std::{
        collections::HashSet,
        env, fmt,
        fmt::Write as _,
        fs::File,
        io::Read,
        path::{Path, PathBuf},
        time::Duration,
    };

    use chrono::{
        DateTime, Days, Duration as ChronoDuration, NaiveDate, NaiveDateTime, TimeZone, Utc,
    };
    use curio_service::database::{Database, DatabaseOptions};
    use sha2::{Digest, Sha256};
    use sqlx::{
        PgPool, SqlitePool,
        sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    };

    const IMPORTER_VERSION: &str = env!("CARGO_PKG_VERSION");
    const IMPORTER_COMMIT: &str = match option_env!("CURIO_GIT_COMMIT") {
        Some(commit) => commit,
        None => "unknown",
    };
    const TARGET_MAX_CONNECTIONS: u32 = 2;
    const TARGET_ACQUIRE_TIMEOUT_SECONDS: u64 = 15;
    const ADVISORY_LOCK_NAMESPACE: &str = "curio:sqlite-import";

    const SOURCE_TABLES: [&str; 5] = [
        "_sqlx_migrations",
        "calendar_events",
        "conversations",
        "messages",
        "users",
    ];
    const TARGET_TABLES: [&str; 6] = [
        "_sqlx_migrations",
        "calendar_events",
        "conversations",
        "messages",
        "sqlite_import_manifests",
        "users",
    ];

    type Result<T> = std::result::Result<T, ImportError>;

    #[derive(Debug)]
    pub(super) struct ImportError(&'static str);

    impl ImportError {
        const fn new(message: &'static str) -> Self {
            Self(message)
        }
    }

    impl fmt::Display for ImportError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.0)
        }
    }

    impl std::error::Error for ImportError {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Arguments {
        source: PathBuf,
        target_url_env: String,
        target_schema: String,
        apply: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Counts {
        users: i64,
        conversations: i64,
        messages: i64,
        calendar_events: i64,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TableHashes {
        users: [u8; 32],
        conversations: [u8; 32],
        messages: [u8; 32],
        calendar_events: [u8; 32],
    }

    #[derive(Debug)]
    struct SourceData {
        users: Vec<LegacyUser>,
        conversations: Vec<LegacyConversation>,
        messages: Vec<LegacyMessage>,
        calendar_events: Vec<LegacyCalendarEvent>,
    }

    impl SourceData {
        fn counts(&self) -> Result<Counts> {
            Ok(Counts {
                users: count(self.users.len())?,
                conversations: count(self.conversations.len())?,
                messages: count(self.messages.len())?,
                calendar_events: count(self.calendar_events.len())?,
            })
        }
    }

    #[derive(Debug, sqlx::FromRow)]
    struct LegacyUser {
        id: String,
        name: String,
        email: String,
        avatar_url: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    #[derive(Debug, sqlx::FromRow)]
    struct LegacyConversation {
        id: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    #[derive(Debug, sqlx::FromRow)]
    struct LegacyMessage {
        sort_order: i64,
        id: String,
        conversation_id: String,
        role: String,
        content: String,
        status: String,
        response_id: Option<String>,
        error_code: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    #[derive(Debug, sqlx::FromRow)]
    struct LegacyCalendarEvent {
        id: String,
        user_id: String,
        title: String,
        description: Option<String>,
        status: Option<String>,
        priority: Option<String>,
        all_day: bool,
        start_date: String,
        end_date: Option<String>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    #[derive(sqlx::FromRow)]
    struct RawUser {
        id: String,
        name: String,
        email: String,
        avatar_url: Option<String>,
        created_at: String,
        updated_at: String,
    }

    #[derive(sqlx::FromRow)]
    struct RawConversation {
        id: String,
        created_at: String,
        updated_at: String,
    }

    #[derive(sqlx::FromRow)]
    struct RawMessage {
        sort_order: i64,
        id: String,
        conversation_id: String,
        role: String,
        content: String,
        status: String,
        response_id: Option<String>,
        error_code: Option<String>,
        created_at: String,
        updated_at: String,
    }

    #[derive(sqlx::FromRow)]
    struct RawCalendarEvent {
        id: String,
        user_id: String,
        title: String,
        description: Option<String>,
        status: Option<String>,
        priority: Option<String>,
        all_day: i64,
        start_date: String,
        end_date: Option<String>,
        starts_at: String,
        ends_at: String,
        created_at: String,
        updated_at: String,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TargetState {
        Empty,
        Ready,
    }

    pub async fn run() -> Result<()> {
        let arguments = parse_arguments(env::args().skip(1))?;
        validate_source_path(&arguments.source)?;
        reject_sqlite_sidecars(&arguments.source)?;

        let initial_hash = hash_file(&arguments.source)?;
        let source_pool = connect_source(&arguments.source).await?;
        let source = load_and_validate_source(&source_pool).await;
        source_pool.close().await;
        let source = source?;

        reject_sqlite_sidecars(&arguments.source)?;
        let final_hash = hash_file(&arguments.source)?;
        if initial_hash != final_hash {
            return Err(ImportError::new(
                "the SQLite source changed during preflight; import a stable backup snapshot",
            ));
        }
        let source_sha256 = hex_digest(initial_hash);
        let counts = source.counts()?;
        let source_table_hashes = canonical_table_hashes(&source)?;

        let target_url = env::var(&arguments.target_url_env)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| ImportError::new("the target URL environment variable is not set"))?;

        let database = Database::connect(&DatabaseOptions {
            url: target_url,
            schema: arguments.target_schema.clone(),
            max_connections: TARGET_MAX_CONNECTIONS,
            acquire_timeout: Duration::from_secs(TARGET_ACQUIRE_TIMEOUT_SECONDS),
            application_name: "curio-sqlite-import".to_owned(),
        })
        .await
        .map_err(|_| ImportError::new("could not connect to the PostgreSQL target"))?;

        verify_target_identity(database.pool(), &arguments.target_schema).await?;
        let initial_target_state =
            inspect_target_catalog(database.pool(), &arguments.target_schema).await?;

        if arguments.apply {
            database
                .migrate()
                .await
                .map_err(|_| ImportError::new("PostgreSQL migrations could not be applied"))?;
            database
                .verify_migrations()
                .await
                .map_err(|_| ImportError::new("the PostgreSQL migration state is not current"))?;

            if inspect_target_catalog(database.pool(), &arguments.target_schema).await?
                != TargetState::Ready
            {
                return Err(ImportError::new(
                    "the PostgreSQL target does not contain the exact expected Curio schema",
                ));
            }

            verify_target_empty(database.pool()).await?;
            apply_import(
                database.pool(),
                &arguments.target_schema,
                &source,
                counts,
                &source_sha256,
                &source_table_hashes,
            )
            .await?;
            verify_target_counts(database.pool(), counts).await?;
            verify_target_hashes(database.pool(), &source_table_hashes).await?;
            verify_manifest(database.pool(), &source_sha256, counts).await?;

            println!("SQLite import completed and verified.");
            print_summary(&source_sha256, counts);
        } else {
            match initial_target_state {
                TargetState::Ready => {
                    database.verify_migrations().await.map_err(|_| {
                        ImportError::new("the PostgreSQL migration state is not current")
                    })?;
                    verify_target_empty(database.pool()).await?;
                }
                TargetState::Empty => {}
            }

            println!("SQLite import dry run passed; no target writes were made.");
            if initial_target_state == TargetState::Empty {
                println!(
                    "The target schema is empty; apply mode will run the embedded migrations first."
                );
            }
            print_summary(&source_sha256, counts);
        }

        database.pool().close().await;
        Ok(())
    }

    fn parse_arguments(arguments: impl IntoIterator<Item = String>) -> Result<Arguments> {
        let mut source = None;
        let mut target_url_env = None;
        let mut target_schema = None;
        let mut apply = false;
        let mut arguments = arguments.into_iter();

        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--source" if source.is_none() => {
                    source = Some(required_argument_value(arguments.next())?);
                }
                "--target-url-env" if target_url_env.is_none() => {
                    target_url_env = Some(required_argument_value(arguments.next())?);
                }
                "--target-schema" if target_schema.is_none() => {
                    target_schema = Some(required_argument_value(arguments.next())?);
                }
                "--apply" if !apply => apply = true,
                "--help" | "-h" => {
                    return Err(ImportError::new(
                        "usage: import_sqlite --source PATH --target-url-env ENV --target-schema SCHEMA [--apply]",
                    ));
                }
                _ => {
                    return Err(ImportError::new(
                        "invalid arguments; use --source, --target-url-env, --target-schema, and optionally --apply",
                    ));
                }
            }
        }

        let source = source.ok_or_else(|| ImportError::new("--source is required"))?;
        let target_url_env =
            target_url_env.ok_or_else(|| ImportError::new("--target-url-env is required"))?;
        let target_schema =
            target_schema.ok_or_else(|| ImportError::new("--target-schema is required"))?;

        if !valid_environment_variable_name(&target_url_env) {
            return Err(ImportError::new(
                "--target-url-env must be an environment variable name, not a connection string",
            ));
        }
        if !allowed_target_schema(&target_schema) {
            return Err(ImportError::new(
                "target schema must be curio_dev, curio_prod, or a curio_test_<run-id> schema",
            ));
        }

        Ok(Arguments {
            source: PathBuf::from(source),
            target_url_env,
            target_schema,
            apply,
        })
    }

    fn required_argument_value(value: Option<String>) -> Result<String> {
        value
            .filter(|value| !value.trim().is_empty() && !value.starts_with("--"))
            .ok_or_else(|| ImportError::new("an option is missing its value"))
    }

    fn valid_environment_variable_name(value: &str) -> bool {
        let mut bytes = value.bytes();
        let Some(first) = bytes.next() else {
            return false;
        };

        (first == b'_' || first.is_ascii_uppercase())
            && bytes.all(|byte| byte == b'_' || byte.is_ascii_uppercase() || byte.is_ascii_digit())
    }

    fn allowed_target_schema(value: &str) -> bool {
        if matches!(value, "curio_dev" | "curio_prod") {
            return true;
        }

        let Some(suffix) = value.strip_prefix("curio_test_") else {
            return false;
        };
        let mut bytes = suffix.bytes();
        let Some(first) = bytes.next() else {
            return false;
        };

        (first.is_ascii_lowercase() || first.is_ascii_digit())
            && bytes.all(|byte| byte == b'_' || byte.is_ascii_lowercase() || byte.is_ascii_digit())
    }

    fn validate_source_path(path: &Path) -> Result<()> {
        let metadata = path
            .metadata()
            .map_err(|_| ImportError::new("the SQLite source could not be inspected"))?;
        if !metadata.is_file() {
            return Err(ImportError::new("the SQLite source must be a regular file"));
        }
        Ok(())
    }

    fn reject_sqlite_sidecars(path: &Path) -> Result<()> {
        for suffix in ["-wal", "-shm", "-journal"] {
            let mut sidecar = path.as_os_str().to_owned();
            sidecar.push(suffix);
            if PathBuf::from(sidecar).exists() {
                return Err(ImportError::new(
                    "the SQLite source has a WAL or journal sidecar; create a consistent backup first",
                ));
            }
        }
        Ok(())
    }

    fn hash_file(path: &Path) -> Result<[u8; 32]> {
        let mut file = File::open(path)
            .map_err(|_| ImportError::new("the SQLite source could not be opened for hashing"))?;
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];

        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|_| ImportError::new("the SQLite source could not be hashed"))?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }

        Ok(hasher.finalize().into())
    }

    fn hex_digest(digest: [u8; 32]) -> String {
        let mut encoded = String::with_capacity(64);
        for byte in digest {
            write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
        }
        encoded
    }

    async fn connect_source(path: &Path) -> Result<SqlitePool> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .read_only(true)
            .create_if_missing(false)
            .immutable(true)
            .foreign_keys(true);

        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(|_| ImportError::new("the SQLite source could not be opened read-only"))
    }

    async fn load_and_validate_source(pool: &SqlitePool) -> Result<SourceData> {
        verify_source_integrity(pool).await?;
        verify_source_catalog(pool).await?;

        let users = load_users(pool).await?;
        let conversations = load_conversations(pool).await?;
        let messages = load_messages(pool).await?;
        let calendar_events = load_calendar_events(pool).await?;

        validate_source_relationships(&users, &conversations, &messages, &calendar_events)?;

        Ok(SourceData {
            users,
            conversations,
            messages,
            calendar_events,
        })
    }

    async fn verify_source_integrity(pool: &SqlitePool) -> Result<()> {
        let results = sqlx::query_scalar::<_, String>("PRAGMA integrity_check")
            .fetch_all(pool)
            .await
            .map_err(|_| ImportError::new("SQLite integrity_check could not run"))?;
        if results.as_slice() != ["ok"] {
            return Err(ImportError::new("SQLite integrity_check did not pass"));
        }

        if sqlx::query("PRAGMA foreign_key_check")
            .fetch_optional(pool)
            .await
            .map_err(|_| ImportError::new("SQLite foreign_key_check could not run"))?
            .is_some()
        {
            return Err(ImportError::new(
                "SQLite foreign_key_check found violations",
            ));
        }

        Ok(())
    }

    async fn verify_source_catalog(pool: &SqlitePool) -> Result<()> {
        let mut objects = sqlx::query_as::<_, (String, String)>(
            r#"
            SELECT name, type
            FROM sqlite_master
            WHERE type IN ('table', 'view', 'trigger')
              AND name NOT LIKE 'sqlite_%'
            ORDER BY name
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|_| ImportError::new("the SQLite catalog could not be inspected"))?;

        if objects.iter().any(|(_, kind)| kind != "table") {
            return Err(ImportError::new(
                "the SQLite source contains an unrecognized view or trigger",
            ));
        }
        let mut names = objects.drain(..).map(|(name, _)| name).collect::<Vec<_>>();
        names.sort();
        if names != SOURCE_TABLES {
            return Err(ImportError::new(
                "the SQLite source does not contain the exact expected Curio tables",
            ));
        }

        let migrations = sqlx::query_as::<_, (i64, i64)>(
            "SELECT version, success FROM _sqlx_migrations ORDER BY version",
        )
        .fetch_all(pool)
        .await
        .map_err(|_| ImportError::new("the SQLite migration history could not be inspected"))?;
        if migrations != [(1, 1), (2, 1), (3, 1)] {
            return Err(ImportError::new(
                "the SQLite source migration history is not the expected version 1-3 baseline",
            ));
        }

        Ok(())
    }

    async fn load_users(pool: &SqlitePool) -> Result<Vec<LegacyUser>> {
        let rows = sqlx::query_as::<_, RawUser>(
            r#"
            SELECT id, name, email, avatar_url, created_at, updated_at
            FROM users
            ORDER BY id
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|_| ImportError::new("users could not be read from the SQLite source"))?;

        rows.into_iter()
            .map(|row| {
                Ok(LegacyUser {
                    id: row.id,
                    name: row.name,
                    email: row.email,
                    avatar_url: row.avatar_url,
                    created_at: parse_legacy_timestamp(&row.created_at)?,
                    updated_at: parse_legacy_timestamp(&row.updated_at)?,
                })
            })
            .collect()
    }

    async fn load_conversations(pool: &SqlitePool) -> Result<Vec<LegacyConversation>> {
        let rows = sqlx::query_as::<_, RawConversation>(
            "SELECT id, created_at, updated_at FROM conversations ORDER BY id",
        )
        .fetch_all(pool)
        .await
        .map_err(|_| ImportError::new("conversations could not be read from the SQLite source"))?;

        rows.into_iter()
            .map(|row| {
                Ok(LegacyConversation {
                    id: row.id,
                    created_at: parse_legacy_timestamp(&row.created_at)?,
                    updated_at: parse_legacy_timestamp(&row.updated_at)?,
                })
            })
            .collect()
    }

    async fn load_messages(pool: &SqlitePool) -> Result<Vec<LegacyMessage>> {
        let rows = sqlx::query_as::<_, RawMessage>(
            r#"
            SELECT rowid AS sort_order, id, conversation_id, role, content, status,
                   response_id, error_code, created_at, updated_at
            FROM messages
            ORDER BY rowid
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|_| ImportError::new("messages could not be read from the SQLite source"))?;

        rows.into_iter()
            .map(|row| {
                if row.sort_order <= 0 {
                    return Err(ImportError::new(
                        "a message has a non-positive SQLite rowid",
                    ));
                }
                if !matches!(row.role.as_str(), "user" | "assistant") {
                    return Err(ImportError::new("a message has an invalid role"));
                }
                if !matches!(
                    row.status.as_str(),
                    "pending" | "completed" | "failed" | "interrupted"
                ) {
                    return Err(ImportError::new("a message has an invalid status"));
                }

                Ok(LegacyMessage {
                    sort_order: row.sort_order,
                    id: row.id,
                    conversation_id: row.conversation_id,
                    role: row.role,
                    content: row.content,
                    status: row.status,
                    response_id: row.response_id,
                    error_code: row.error_code,
                    created_at: parse_legacy_timestamp(&row.created_at)?,
                    updated_at: parse_legacy_timestamp(&row.updated_at)?,
                })
            })
            .collect()
    }

    async fn load_calendar_events(pool: &SqlitePool) -> Result<Vec<LegacyCalendarEvent>> {
        let rows = sqlx::query_as::<_, RawCalendarEvent>(
            r#"
            SELECT id, user_id, title, description, status, priority, all_day,
                   start_date, end_date, starts_at, ends_at, created_at, updated_at
            FROM calendar_events
            ORDER BY id
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|_| {
            ImportError::new("calendar events could not be read from the SQLite source")
        })?;

        rows.into_iter()
            .map(|row| {
                if !matches!(row.all_day, 0 | 1) {
                    return Err(ImportError::new(
                        "a calendar event has an invalid all_day value",
                    ));
                }
                if row.status.as_deref().is_some_and(|status| {
                    !matches!(
                        status,
                        "scheduled" | "in-progress" | "blocked" | "done" | "cancelled"
                    )
                }) {
                    return Err(ImportError::new("a calendar event has an invalid status"));
                }
                if row
                    .priority
                    .as_deref()
                    .is_some_and(|priority| !matches!(priority, "low" | "medium" | "high"))
                {
                    return Err(ImportError::new("a calendar event has an invalid priority"));
                }

                let starts_at = parse_rfc3339_millis(&row.starts_at)?;
                let ends_at = parse_rfc3339_millis(&row.ends_at)?;
                validate_calendar_interval(
                    row.all_day == 1,
                    &row.start_date,
                    row.end_date.as_deref(),
                    starts_at,
                    ends_at,
                )?;

                Ok(LegacyCalendarEvent {
                    id: row.id,
                    user_id: row.user_id,
                    title: row.title,
                    description: row.description,
                    status: row.status,
                    priority: row.priority,
                    all_day: row.all_day == 1,
                    start_date: row.start_date,
                    end_date: row.end_date,
                    starts_at,
                    ends_at,
                    created_at: parse_legacy_timestamp(&row.created_at)?,
                    updated_at: parse_legacy_timestamp(&row.updated_at)?,
                })
            })
            .collect()
    }

    fn parse_legacy_timestamp(value: &str) -> Result<DateTime<Utc>> {
        if let Ok(timestamp) = DateTime::parse_from_rfc3339(value) {
            return require_millisecond_precision(timestamp.with_timezone(&Utc));
        }

        let timestamp = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f")
            .map_err(|_| ImportError::new("a stored timestamp is invalid"))?;
        require_millisecond_precision(Utc.from_utc_datetime(&timestamp))
    }

    fn parse_rfc3339_millis(value: &str) -> Result<DateTime<Utc>> {
        let timestamp = DateTime::parse_from_rfc3339(value)
            .map_err(|_| ImportError::new("a normalized calendar timestamp is invalid"))?;
        require_millisecond_precision(timestamp.with_timezone(&Utc))
    }

    fn require_millisecond_precision(timestamp: DateTime<Utc>) -> Result<DateTime<Utc>> {
        if !timestamp.timestamp_subsec_nanos().is_multiple_of(1_000_000) {
            return Err(ImportError::new(
                "a stored timestamp is more precise than the PostgreSQL millisecond contract",
            ));
        }
        Ok(timestamp)
    }

    fn parse_date(value: &str) -> Option<NaiveDate> {
        let bytes = value.as_bytes();
        let shaped = bytes.len() == 10
            && bytes[4] == b'-'
            && bytes[7] == b'-'
            && [0, 1, 2, 3, 5, 6, 8, 9]
                .iter()
                .all(|index| bytes[*index].is_ascii_digit());
        shaped
            .then(|| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
            .flatten()
    }

    fn validate_calendar_interval(
        all_day: bool,
        start_date: &str,
        end_date: Option<&str>,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    ) -> Result<()> {
        if ends_at <= starts_at {
            return Err(ImportError::new(
                "a calendar event has an invalid normalized interval",
            ));
        }

        let (expected_start, expected_end) = if all_day {
            let start = parse_date(start_date)
                .ok_or_else(|| ImportError::new("an all-day calendar event has an invalid date"))?;
            let end = match end_date {
                Some(value) => parse_date(value).ok_or_else(|| {
                    ImportError::new("an all-day calendar event has an invalid end date")
                })?,
                None => start.checked_add_days(Days::new(1)).ok_or_else(|| {
                    ImportError::new("an all-day calendar event date is out of range")
                })?,
            };
            if end <= start {
                return Err(ImportError::new(
                    "an all-day calendar event has a non-positive interval",
                ));
            }
            (
                Utc.from_utc_datetime(&start.and_hms_opt(0, 0, 0).expect("midnight is valid")),
                Utc.from_utc_datetime(&end.and_hms_opt(0, 0, 0).expect("midnight is valid")),
            )
        } else {
            let start = parse_rfc3339_millis(start_date)?;
            let end = match end_date {
                Some(value) => parse_rfc3339_millis(value)?,
                None => start
                    .checked_add_signed(ChronoDuration::minutes(1))
                    .ok_or_else(|| {
                        ImportError::new("a timed calendar event date is out of range")
                    })?,
            };
            if end <= start {
                return Err(ImportError::new(
                    "a timed calendar event has a non-positive interval",
                ));
            }
            (start, end)
        };

        if starts_at != expected_start || ends_at != expected_end {
            return Err(ImportError::new(
                "a calendar event's wire dates do not match its normalized interval",
            ));
        }
        Ok(())
    }

    fn validate_source_relationships(
        users: &[LegacyUser],
        conversations: &[LegacyConversation],
        messages: &[LegacyMessage],
        calendar_events: &[LegacyCalendarEvent],
    ) -> Result<()> {
        ensure_unique(
            users.iter().map(|row| row.id.as_str()),
            "duplicate user ids",
        )?;
        ensure_unique(
            users.iter().map(|row| row.email.as_str()),
            "duplicate user emails",
        )?;
        ensure_unique(
            conversations.iter().map(|row| row.id.as_str()),
            "duplicate conversation ids",
        )?;
        ensure_unique(
            messages.iter().map(|row| row.id.as_str()),
            "duplicate message ids",
        )?;
        ensure_unique(
            messages.iter().map(|row| row.sort_order),
            "duplicate message rowids",
        )?;
        ensure_unique(
            calendar_events.iter().map(|row| row.id.as_str()),
            "duplicate calendar event ids",
        )?;

        let user_ids = users
            .iter()
            .map(|row| row.id.as_str())
            .collect::<HashSet<_>>();
        let conversation_ids = conversations
            .iter()
            .map(|row| row.id.as_str())
            .collect::<HashSet<_>>();

        if messages
            .iter()
            .any(|row| !conversation_ids.contains(row.conversation_id.as_str()))
        {
            return Err(ImportError::new(
                "a message references an unknown conversation",
            ));
        }
        if calendar_events
            .iter()
            .any(|row| !user_ids.contains(row.user_id.as_str()))
        {
            return Err(ImportError::new(
                "a calendar event references an unknown user",
            ));
        }
        Ok(())
    }

    fn ensure_unique<T>(values: impl IntoIterator<Item = T>, message: &'static str) -> Result<()>
    where
        T: Eq + std::hash::Hash,
    {
        let mut seen = HashSet::new();
        if values.into_iter().any(|value| !seen.insert(value)) {
            return Err(ImportError::new(message));
        }
        Ok(())
    }

    fn canonical_table_hashes(data: &SourceData) -> Result<TableHashes> {
        let mut users = CanonicalHasher::new("users", data.users.len())?;
        for row in &data.users {
            users.start_row();
            users.string(&row.id)?;
            users.string(&row.name)?;
            users.string(&row.email)?;
            users.optional_string(row.avatar_url.as_deref())?;
            users.timestamp(row.created_at)?;
            users.timestamp(row.updated_at)?;
        }

        let mut conversations = CanonicalHasher::new("conversations", data.conversations.len())?;
        for row in &data.conversations {
            conversations.start_row();
            conversations.string(&row.id)?;
            conversations.timestamp(row.created_at)?;
            conversations.timestamp(row.updated_at)?;
        }

        let mut messages = CanonicalHasher::new("messages", data.messages.len())?;
        for row in &data.messages {
            messages.start_row();
            messages.integer(row.sort_order)?;
            messages.string(&row.id)?;
            messages.string(&row.conversation_id)?;
            messages.string(&row.role)?;
            messages.string(&row.content)?;
            messages.string(&row.status)?;
            messages.optional_string(row.response_id.as_deref())?;
            messages.optional_string(row.error_code.as_deref())?;
            messages.timestamp(row.created_at)?;
            messages.timestamp(row.updated_at)?;
        }

        let mut calendar_events =
            CanonicalHasher::new("calendar_events", data.calendar_events.len())?;
        for row in &data.calendar_events {
            calendar_events.start_row();
            calendar_events.string(&row.id)?;
            calendar_events.string(&row.user_id)?;
            calendar_events.string(&row.title)?;
            calendar_events.optional_string(row.description.as_deref())?;
            calendar_events.optional_string(row.status.as_deref())?;
            calendar_events.optional_string(row.priority.as_deref())?;
            calendar_events.boolean(row.all_day)?;
            calendar_events.string(&row.start_date)?;
            calendar_events.optional_string(row.end_date.as_deref())?;
            calendar_events.timestamp(row.starts_at)?;
            calendar_events.timestamp(row.ends_at)?;
            calendar_events.timestamp(row.created_at)?;
            calendar_events.timestamp(row.updated_at)?;
        }

        Ok(TableHashes {
            users: users.finish(),
            conversations: conversations.finish(),
            messages: messages.finish(),
            calendar_events: calendar_events.finish(),
        })
    }

    struct CanonicalHasher {
        hasher: Sha256,
    }

    impl CanonicalHasher {
        fn new(table: &str, rows: usize) -> Result<Self> {
            let mut canonical = Self {
                hasher: Sha256::new(),
            };
            canonical.value(Some(b"curio-sqlite-import-canonical-v1"))?;
            canonical.value(Some(table.as_bytes()))?;
            let rows = u64::try_from(rows)
                .map_err(|_| ImportError::new("a canonical hash row count is too large"))?;
            canonical.value(Some(&rows.to_be_bytes()))?;
            Ok(canonical)
        }

        fn start_row(&mut self) {
            self.hasher.update([0x52]);
        }

        fn string(&mut self, value: &str) -> Result<()> {
            self.value(Some(value.as_bytes()))
        }

        fn optional_string(&mut self, value: Option<&str>) -> Result<()> {
            self.value(value.map(str::as_bytes))
        }

        fn timestamp(&mut self, value: DateTime<Utc>) -> Result<()> {
            self.value(Some(&value.timestamp_millis().to_be_bytes()))
        }

        fn integer(&mut self, value: i64) -> Result<()> {
            self.value(Some(&value.to_be_bytes()))
        }

        fn boolean(&mut self, value: bool) -> Result<()> {
            self.value(Some(&[u8::from(value)]))
        }

        fn value(&mut self, value: Option<&[u8]>) -> Result<()> {
            let Some(value) = value else {
                self.hasher.update([0x00]);
                return Ok(());
            };

            self.hasher.update([0x01]);
            let length = u64::try_from(value.len())
                .map_err(|_| ImportError::new("a canonical hash field is too large"))?;
            self.hasher.update(length.to_be_bytes());
            self.hasher.update(value);
            Ok(())
        }

        fn finish(self) -> [u8; 32] {
            self.hasher.finalize().into()
        }
    }

    async fn load_target_data(pool: &PgPool) -> Result<SourceData> {
        Ok(SourceData {
            users: load_target_users(pool).await?,
            conversations: load_target_conversations(pool).await?,
            messages: load_target_messages(pool).await?,
            calendar_events: load_target_calendar_events(pool).await?,
        })
    }

    async fn load_target_data_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<SourceData> {
        Ok(SourceData {
            users: load_target_users(&mut **transaction).await?,
            conversations: load_target_conversations(&mut **transaction).await?,
            messages: load_target_messages(&mut **transaction).await?,
            calendar_events: load_target_calendar_events(&mut **transaction).await?,
        })
    }

    async fn load_target_users<'executor, E>(executor: E) -> Result<Vec<LegacyUser>>
    where
        E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
    {
        sqlx::query_as::<_, LegacyUser>(
            r#"
            SELECT id, name, email, avatar_url, created_at, updated_at
            FROM users
            ORDER BY id
            "#,
        )
        .fetch_all(executor)
        .await
        .map_err(|_| ImportError::new("PostgreSQL users could not be read for hash verification"))
    }

    async fn load_target_conversations<'executor, E>(executor: E) -> Result<Vec<LegacyConversation>>
    where
        E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
    {
        sqlx::query_as::<_, LegacyConversation>(
            "SELECT id, created_at, updated_at FROM conversations ORDER BY id",
        )
        .fetch_all(executor)
        .await
        .map_err(|_| {
            ImportError::new("PostgreSQL conversations could not be read for hash verification")
        })
    }

    async fn load_target_messages<'executor, E>(executor: E) -> Result<Vec<LegacyMessage>>
    where
        E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
    {
        sqlx::query_as::<_, LegacyMessage>(
            r#"
            SELECT sort_order, id, conversation_id, role, content, status,
                   response_id, error_code, created_at, updated_at
            FROM messages
            ORDER BY sort_order, id
            "#,
        )
        .fetch_all(executor)
        .await
        .map_err(|_| {
            ImportError::new("PostgreSQL messages could not be read for hash verification")
        })
    }

    async fn load_target_calendar_events<'executor, E>(
        executor: E,
    ) -> Result<Vec<LegacyCalendarEvent>>
    where
        E: sqlx::Executor<'executor, Database = sqlx::Postgres>,
    {
        sqlx::query_as::<_, LegacyCalendarEvent>(
            r#"
            SELECT id, user_id, title, description, status, priority, all_day,
                   start_date, end_date, starts_at, ends_at, created_at, updated_at
            FROM calendar_events
            ORDER BY id
            "#,
        )
        .fetch_all(executor)
        .await
        .map_err(|_| {
            ImportError::new("PostgreSQL calendar events could not be read for hash verification")
        })
    }

    async fn verify_target_identity(pool: &PgPool, schema: &str) -> Result<()> {
        let (database, user, current_schema) =
            sqlx::query_as::<_, (String, String, Option<String>)>(
                "SELECT current_database(), current_user, current_schema()",
            )
            .fetch_one(pool)
            .await
            .map_err(|_| {
                ImportError::new("the PostgreSQL target identity could not be verified")
            })?;

        if database.trim().is_empty()
            || user.trim().is_empty()
            || current_schema.as_deref() != Some(schema)
        {
            return Err(ImportError::new(
                "the PostgreSQL connection is not using the requested target schema",
            ));
        }
        Ok(())
    }

    async fn inspect_target_catalog(pool: &PgPool, schema: &str) -> Result<TargetState> {
        let relations = sqlx::query_as::<_, (String, String)>(
            r#"
            SELECT class.relname, class.relkind::text
            FROM pg_catalog.pg_class AS class
            JOIN pg_catalog.pg_namespace AS namespace
              ON namespace.oid = class.relnamespace
            WHERE namespace.nspname = $1
              AND class.relkind IN ('r', 'p', 'v', 'm', 'f', 'S')
            ORDER BY class.relname
            "#,
        )
        .bind(schema)
        .fetch_all(pool)
        .await
        .map_err(|_| ImportError::new("the PostgreSQL target catalog could not be inspected"))?;

        let has_routines = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM information_schema.routines
                WHERE routine_schema = $1
            )
            "#,
        )
        .bind(schema)
        .fetch_one(pool)
        .await
        .map_err(|_| ImportError::new("the PostgreSQL target routines could not be inspected"))?;
        let has_domains_or_enums = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM pg_catalog.pg_type AS type
                JOIN pg_catalog.pg_namespace AS namespace
                  ON namespace.oid = type.typnamespace
                WHERE namespace.nspname = $1
                  AND type.typtype IN ('d', 'e')
            )
            "#,
        )
        .bind(schema)
        .fetch_one(pool)
        .await
        .map_err(|_| ImportError::new("the PostgreSQL target types could not be inspected"))?;

        if has_routines || has_domains_or_enums {
            return Err(ImportError::new(
                "the PostgreSQL target schema contains unrecognized objects",
            ));
        }
        if relations.is_empty() {
            return Ok(TargetState::Empty);
        }

        let expected_relations = TARGET_TABLES
            .iter()
            .map(|name| ((*name).to_owned(), "r".to_owned()))
            .chain(std::iter::once((
                "messages_sort_order_seq".to_owned(),
                "S".to_owned(),
            )))
            .collect::<HashSet<_>>();
        let actual_relations = relations.into_iter().collect::<HashSet<_>>();
        if actual_relations != expected_relations {
            return Err(ImportError::new(
                "the PostgreSQL target schema contains unknown or incomplete relations",
            ));
        }

        Ok(TargetState::Ready)
    }

    async fn verify_target_empty(pool: &PgPool) -> Result<()> {
        if manifest_count(pool).await? != 0 {
            return Err(ImportError::new(
                "the PostgreSQL target already has an SQLite import manifest",
            ));
        }

        let counts = target_counts(pool).await?;
        if counts
            != (Counts {
                users: 0,
                conversations: 0,
                messages: 0,
                calendar_events: 0,
            })
        {
            return Err(ImportError::new(
                "the PostgreSQL target already contains application rows",
            ));
        }
        Ok(())
    }

    async fn apply_import(
        pool: &PgPool,
        schema: &str,
        source: &SourceData,
        counts: Counts,
        source_sha256: &str,
        source_table_hashes: &TableHashes,
    ) -> Result<()> {
        let mut transaction = pool
            .begin()
            .await
            .map_err(|_| ImportError::new("the PostgreSQL import transaction could not start"))?;

        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(format!("{ADVISORY_LOCK_NAMESPACE}:{schema}"))
            .execute(&mut *transaction)
            .await
            .map_err(|_| ImportError::new("the PostgreSQL import lock could not be acquired"))?;
        sqlx::query(
            r#"
            LOCK TABLE users, conversations, messages, calendar_events,
                       sqlite_import_manifests IN ACCESS EXCLUSIVE MODE
            "#,
        )
        .execute(&mut *transaction)
        .await
        .map_err(|_| ImportError::new("the PostgreSQL target tables could not be locked"))?;

        verify_target_empty_in_transaction(&mut transaction).await?;

        for row in &source.users {
            sqlx::query(
                r#"
                INSERT INTO users (
                    id, name, email, avatar_url, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(&row.id)
            .bind(&row.name)
            .bind(&row.email)
            .bind(&row.avatar_url)
            .bind(row.created_at)
            .bind(row.updated_at)
            .execute(&mut *transaction)
            .await
            .map_err(|_| ImportError::new("a user row could not be imported"))?;
        }

        for row in &source.conversations {
            sqlx::query(
                r#"
                INSERT INTO conversations (id, created_at, updated_at)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(&row.id)
            .bind(row.created_at)
            .bind(row.updated_at)
            .execute(&mut *transaction)
            .await
            .map_err(|_| ImportError::new("a conversation row could not be imported"))?;
        }

        for row in &source.messages {
            sqlx::query(
                r#"
                INSERT INTO messages (
                    sort_order, id, conversation_id, role, content, status,
                    response_id, error_code, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(row.sort_order)
            .bind(&row.id)
            .bind(&row.conversation_id)
            .bind(&row.role)
            .bind(&row.content)
            .bind(&row.status)
            .bind(&row.response_id)
            .bind(&row.error_code)
            .bind(row.created_at)
            .bind(row.updated_at)
            .execute(&mut *transaction)
            .await
            .map_err(|_| ImportError::new("a message row could not be imported"))?;
        }

        sqlx::query(
            r#"
            SELECT setval(
                pg_get_serial_sequence('messages', 'sort_order'),
                COALESCE((SELECT MAX(sort_order) FROM messages), 1),
                EXISTS (SELECT 1 FROM messages)
            )
            "#,
        )
        .execute(&mut *transaction)
        .await
        .map_err(|_| ImportError::new("the message identity sequence could not be advanced"))?;

        for row in &source.calendar_events {
            sqlx::query(
                r#"
                INSERT INTO calendar_events (
                    id, user_id, title, description, status, priority, all_day,
                    start_date, end_date, starts_at, ends_at, created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    $8, $9, $10, $11, $12, $13
                )
                "#,
            )
            .bind(&row.id)
            .bind(&row.user_id)
            .bind(&row.title)
            .bind(&row.description)
            .bind(&row.status)
            .bind(&row.priority)
            .bind(row.all_day)
            .bind(&row.start_date)
            .bind(&row.end_date)
            .bind(row.starts_at)
            .bind(row.ends_at)
            .bind(row.created_at)
            .bind(row.updated_at)
            .execute(&mut *transaction)
            .await
            .map_err(|_| ImportError::new("a calendar event row could not be imported"))?;
        }

        let imported_counts = target_counts_with_executor(&mut transaction).await?;
        if imported_counts != counts {
            return Err(ImportError::new(
                "PostgreSQL row counts did not match the SQLite source before commit",
            ));
        }
        let imported_data = load_target_data_in_transaction(&mut transaction).await?;
        if canonical_table_hashes(&imported_data)? != *source_table_hashes {
            return Err(ImportError::new(
                "PostgreSQL canonical row hashes did not match the SQLite source before commit",
            ));
        }

        sqlx::query(
            r#"
            INSERT INTO sqlite_import_manifests (
                source_sha256, completed_at, importer_version, importer_commit,
                users_count, conversations_count, messages_count, calendar_events_count
            ) VALUES ($1, CURRENT_TIMESTAMP, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(source_sha256)
        .bind(IMPORTER_VERSION)
        .bind(IMPORTER_COMMIT)
        .bind(counts.users)
        .bind(counts.conversations)
        .bind(counts.messages)
        .bind(counts.calendar_events)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ImportError::new("the SQLite import manifest could not be recorded"))?;

        transaction
            .commit()
            .await
            .map_err(|_| ImportError::new("the PostgreSQL import transaction could not commit"))
    }

    async fn verify_target_empty_in_transaction(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<()> {
        let manifest =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::bigint FROM sqlite_import_manifests")
                .fetch_one(&mut **transaction)
                .await
                .map_err(|_| ImportError::new("the target import manifest could not be checked"))?;
        if manifest != 0 {
            return Err(ImportError::new(
                "the PostgreSQL target already has an SQLite import manifest",
            ));
        }

        let counts = target_counts_with_executor(transaction).await?;
        if counts
            != (Counts {
                users: 0,
                conversations: 0,
                messages: 0,
                calendar_events: 0,
            })
        {
            return Err(ImportError::new(
                "the PostgreSQL target gained application rows before import",
            ));
        }
        Ok(())
    }

    async fn target_counts(pool: &PgPool) -> Result<Counts> {
        let (users, conversations, messages, calendar_events) =
            sqlx::query_as::<_, (i64, i64, i64, i64)>(
                r#"
                SELECT
                    (SELECT COUNT(*)::bigint FROM users),
                    (SELECT COUNT(*)::bigint FROM conversations),
                    (SELECT COUNT(*)::bigint FROM messages),
                    (SELECT COUNT(*)::bigint FROM calendar_events)
                "#,
            )
            .fetch_one(pool)
            .await
            .map_err(|_| ImportError::new("the PostgreSQL target row counts could not be read"))?;
        Ok(Counts {
            users,
            conversations,
            messages,
            calendar_events,
        })
    }

    async fn target_counts_with_executor(
        transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Counts> {
        let (users, conversations, messages, calendar_events) = sqlx::query_as::<
            _,
            (i64, i64, i64, i64),
        >(
            r#"
                SELECT
                    (SELECT COUNT(*)::bigint FROM users),
                    (SELECT COUNT(*)::bigint FROM conversations),
                    (SELECT COUNT(*)::bigint FROM messages),
                    (SELECT COUNT(*)::bigint FROM calendar_events)
                "#,
        )
        .fetch_one(&mut **transaction)
        .await
        .map_err(|_| ImportError::new("the imported PostgreSQL row counts could not be read"))?;
        Ok(Counts {
            users,
            conversations,
            messages,
            calendar_events,
        })
    }

    async fn manifest_count(pool: &PgPool) -> Result<i64> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::bigint FROM sqlite_import_manifests")
            .fetch_one(pool)
            .await
            .map_err(|_| ImportError::new("the PostgreSQL import manifest could not be checked"))
    }

    async fn verify_target_counts(pool: &PgPool, expected: Counts) -> Result<()> {
        if target_counts(pool).await? != expected {
            return Err(ImportError::new(
                "PostgreSQL row counts did not match after import",
            ));
        }
        Ok(())
    }

    async fn verify_target_hashes(pool: &PgPool, expected: &TableHashes) -> Result<()> {
        let target = load_target_data(pool).await?;
        if canonical_table_hashes(&target)? != *expected {
            return Err(ImportError::new(
                "PostgreSQL canonical row hashes did not match after import",
            ));
        }
        Ok(())
    }

    async fn verify_manifest(pool: &PgPool, source_sha256: &str, counts: Counts) -> Result<()> {
        let stored = sqlx::query_as::<_, (String, i64, i64, i64, i64)>(
            r#"
            SELECT source_sha256, users_count, conversations_count,
                   messages_count, calendar_events_count
            FROM sqlite_import_manifests
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|_| ImportError::new("the completed SQLite import manifest could not be read"))?;

        if stored.as_slice()
            != [(
                source_sha256.to_owned(),
                counts.users,
                counts.conversations,
                counts.messages,
                counts.calendar_events,
            )]
        {
            return Err(ImportError::new(
                "the completed SQLite import manifest did not verify",
            ));
        }
        Ok(())
    }

    fn count(value: usize) -> Result<i64> {
        i64::try_from(value)
            .map_err(|_| ImportError::new("a source row count exceeds PostgreSQL bigint"))
    }

    fn print_summary(source_sha256: &str, counts: Counts) {
        println!("Source SHA-256: {source_sha256}");
        println!(
            "Rows: users={}, conversations={}, messages={}, calendar_events={}",
            counts.users, counts.conversations, counts.messages, counts.calendar_events
        );
        println!(
            "Verification covers the immutable source hash, domains, relationships, per-table counts, and canonical hashes of every imported column."
        );
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn arguments_are_explicit_and_dry_run_by_default() {
            let arguments = parse_arguments([
                "--source".to_owned(),
                "legacy.sqlite".to_owned(),
                "--target-url-env".to_owned(),
                "CURIO_MIGRATOR_DATABASE_URL".to_owned(),
                "--target-schema".to_owned(),
                "curio_dev".to_owned(),
            ])
            .unwrap();

            assert_eq!(arguments.source, PathBuf::from("legacy.sqlite"));
            assert!(!arguments.apply);
        }

        #[test]
        fn target_schema_guard_is_conservative() {
            for allowed in [
                "curio_dev",
                "curio_prod",
                "curio_test_a1",
                "curio_test_2026_08",
            ] {
                assert!(allowed_target_schema(allowed));
            }
            for rejected in [
                "public",
                "prod",
                "curio_production",
                "curio_test_",
                "curio_test_-bad",
                "CURIO_DEV",
            ] {
                assert!(!allowed_target_schema(rejected));
            }
        }

        #[test]
        fn timestamps_accept_sqlite_utc_and_offset_rfc3339() {
            assert_eq!(
                parse_legacy_timestamp("2026-08-31 12:34:56").unwrap(),
                parse_legacy_timestamp("2026-08-31T07:34:56-05:00").unwrap()
            );
            assert!(parse_legacy_timestamp("2026-08-31T12:34:56").is_err());
            assert!(parse_legacy_timestamp("2026-08-31T12:34:56.000001Z").is_err());
        }

        #[test]
        fn calendar_preflight_checks_wire_and_normalized_ranges() {
            let starts_at = parse_rfc3339_millis("2026-06-22T00:00:00.000Z").unwrap();
            let ends_at = parse_rfc3339_millis("2026-06-23T00:00:00.000Z").unwrap();
            assert!(
                validate_calendar_interval(true, "2026-06-22", None, starts_at, ends_at).is_ok()
            );
            assert!(
                validate_calendar_interval(
                    true,
                    "2026-06-22",
                    None,
                    starts_at,
                    ends_at + ChronoDuration::days(1),
                )
                .is_err()
            );
        }

        #[test]
        fn canonical_encoding_distinguishes_boundaries_and_nulls() {
            fn hash(values: &[Option<&str>]) -> [u8; 32] {
                let mut hasher = CanonicalHasher::new("test", 1).unwrap();
                hasher.start_row();
                for value in values {
                    hasher.optional_string(*value).unwrap();
                }
                hasher.finish()
            }

            assert_ne!(hash(&[Some(""), Some("x")]), hash(&[None, Some("x")]));
            assert_ne!(
                hash(&[Some("ab"), Some("c")]),
                hash(&[Some("a"), Some("bc")])
            );
            assert_eq!(hash(&[Some("same"), None]), hash(&[Some("same"), None]));
        }
    }
}
