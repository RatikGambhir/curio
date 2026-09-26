//! Full-router and PostgreSQL contract tests for standalone Spaces.
#[path = "support/postgres.rs"]
mod postgres;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    response::Response,
};
use curio_service::{
    adapters::postgres::client::Database,
    app::config::{DocumentsConfig, ServiceConfig},
    app_with_database,
};
use http_body_util::BodyExt;
use postgres::PostgresFixture;
use serde_json::{Value, json};
use tower::ServiceExt;

fn router(database: Database, fixture: &PostgresFixture) -> Router {
    app_with_database(
        ServiceConfig {
            openai_api_key: "test-only".into(),
            openai_model: "test-only".into(),
            openai_base_url: "http://127.0.0.1:1".into(),
            database_url: fixture.database_url().into(),
            database_schema: fixture.schema().into(),
            database_max_connections: 5,
            database_acquire_timeout_seconds: 5,
            cors_allowed_origins: vec!["http://localhost:5173".into()],
            documents: DocumentsConfig::default(),
        },
        database,
    )
}

async fn setup(name: &str) -> Option<(Router, PostgresFixture)> {
    let fixture = PostgresFixture::provision(name).await?;
    let app = router(fixture.database().clone(), &fixture);
    for owner in ["user-a", "user-b"] {
        let response = request(
            &app,
            "POST",
            "/v1/users",
            Some(owner),
            Some(json!({
                "id":owner, "name":owner, "email":format!("{owner}@example.com")
            })),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
    }
    Some((app, fixture))
}

async fn request(
    app: &Router,
    method: &str,
    path: &str,
    owner: Option<&str>,
    body: Option<Value>,
) -> Response {
    raw_request(
        app,
        method,
        path,
        owner,
        body.map(|value| value.to_string()),
        true,
    )
    .await
}

async fn raw_request(
    app: &Router,
    method: &str,
    path: &str,
    owner: Option<&str>,
    body: Option<String>,
    content_type: bool,
) -> Response {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(owner) = owner {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {owner}"));
    }
    if content_type {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    app.clone()
        .oneshot(
            builder
                .body(body.map(Body::from).unwrap_or_else(Body::empty))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn json_body(response: Response) -> Value {
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

async fn create(app: &Router, owner: &str, name: &str) -> Value {
    let response = request(
        app,
        "POST",
        "/v1/spaces",
        Some(owner),
        Some(json!({"name":name})),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    json_body(response).await
}

async fn list(app: &Router, owner: &str, query: &str) -> Value {
    let response = request(app, "GET", &format!("/v1/spaces{query}"), Some(owner), None).await;
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await
}

async fn assert_error(response: Response, status: StatusCode) -> Value {
    assert_eq!(response.status(), status);
    let body = json_body(response).await;
    assert_eq!(body.as_object().unwrap().len(), 1);
    assert!(
        body["error"]
            .as_str()
            .is_some_and(|message| !message.is_empty())
    );
    body
}

#[tokio::test]
async fn spaces_auth_validation_and_unprovisioned_users() {
    let Some((app, fixture)) = setup("spaces_auth_validation").await else {
        return;
    };
    for (method, path) in [
        ("POST", "/v1/spaces"),
        ("GET", "/v1/spaces"),
        ("DELETE", "/v1/spaces/missing"),
    ] {
        for owner in [None, Some(""), Some("contains whitespace")] {
            let response = request(&app, method, path, owner, None).await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert!(
                response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes()
                    .is_empty()
            );
        }
    }
    for owner in ["user-a", "unprovisioned"] {
        assert_eq!(
            list(&app, owner, "").await,
            json!({"spaces":[],"nextCursor":null})
        );
    }
    assert_error(
        request(
            &app,
            "POST",
            "/v1/spaces",
            Some("unprovisioned"),
            Some(json!({"name":"Research"})),
        )
        .await,
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    assert_error(
        request(
            &app,
            "DELETE",
            "/v1/spaces/missing",
            Some("unprovisioned"),
            None,
        )
        .await,
        StatusCode::NOT_FOUND,
    )
    .await;
    for body in [
        json!({}),
        json!({"name":null}),
        json!({"name":123}),
        json!({"name":"   "}),
        json!({"name":"a".repeat(121)}),
        json!({"name":"x\n"}),
        json!({"name":"x","description":"a".repeat(2001)}),
        json!({"name":"x","description":"\ttext"}),
        json!({"name":"x","ownerId":"user-b"}),
        json!({"name":"x","userId":"user-b"}),
        json!({"name":"x","id":"supplied"}),
    ] {
        assert_error(
            request(&app, "POST", "/v1/spaces", Some("user-a"), Some(body)).await,
            StatusCode::UNPROCESSABLE_ENTITY,
        )
        .await;
    }
    assert_error(
        raw_request(
            &app,
            "POST",
            "/v1/spaces",
            Some("user-a"),
            Some("{".into()),
            true,
        )
        .await,
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_error(
        raw_request(
            &app,
            "POST",
            "/v1/spaces",
            Some("user-a"),
            Some("{}".into()),
            false,
        )
        .await,
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
    )
    .await;
    assert_error(
        request(
            &app,
            "POST",
            "/v1/spaces",
            Some("user-a"),
            Some(json!({"name":"a".repeat(17000)})),
        )
        .await,
        StatusCode::PAYLOAD_TOO_LARGE,
    )
    .await;
    for query in [
        "?limit=0",
        "?limit=101",
        "?limit=-1",
        "?limit=no",
        "?limit=1&limit=2",
        "?cursor=",
        "?cursor=not-base64!",
        "?ownerId=user-b",
        "?userId=user-b",
    ] {
        assert_error(
            request(
                &app,
                "GET",
                &format!("/v1/spaces{query}"),
                Some("user-a"),
                None,
            )
            .await,
            StatusCode::BAD_REQUEST,
        )
        .await;
    }
    // Validation failures must not insert anything.
    assert_eq!(list(&app, "user-a", "").await["spaces"], json!([]));
    fixture.cleanup().await;
}

#[tokio::test]
async fn spaces_create_returns_committed_wire_values_and_scopes_uniqueness() {
    let Some((app, fixture)) = setup("spaces_create").await else {
        return;
    };
    let response = request(
        &app,
        "POST",
        "/v1/spaces",
        Some("user-a"),
        Some(json!({"name":"  Research  ", "description":"  Notes  "})),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let space = json_body(response).await;
    assert_eq!(space.as_object().unwrap().len(), 5);
    assert_eq!(space["name"], "Research");
    assert_eq!(space["description"], "Notes");
    assert_eq!(
        uuid::Uuid::parse_str(space["id"].as_str().unwrap())
            .unwrap()
            .get_version_num(),
        4
    );
    assert_eq!(space["createdAt"], space["updatedAt"]);
    let timestamp = space["createdAt"].as_str().unwrap();
    assert_eq!(timestamp.len(), 24);
    assert!(timestamp.ends_with('Z'));
    chrono::DateTime::parse_from_rfc3339(timestamp).unwrap();
    assert_eq!(list(&app, "user-a", "").await["spaces"], json!([space]));
    for name in ["Research", "research", " RESEARCH "] {
        assert_error(
            request(
                &app,
                "POST",
                "/v1/spaces",
                Some("user-a"),
                Some(json!({"name":name})),
            )
            .await,
            StatusCode::CONFLICT,
        )
        .await;
    }
    let other = create(&app, "user-b", "Research").await;
    assert_ne!(other["id"], space["id"]);
    assert_eq!(other["description"], Value::Null);
    assert_eq!(list(&app, "user-b", "").await["spaces"], json!([other]));
    let response = request(
        &app,
        "POST",
        "/v1/spaces",
        Some("user-a"),
        Some(json!({"name":"é".repeat(120),"description":"é".repeat(2000)})),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let response = request(
        &app,
        "POST",
        "/v1/spaces",
        Some("user-a"),
        Some(json!({"name":"Nullable","description":null})),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(json_body(response).await["description"], Value::Null);
    fixture.cleanup().await;
}

#[tokio::test]
async fn spaces_keyset_pagination_handles_ties_deleted_cursor_and_foreign_cursor() {
    let Some((app, fixture)) = setup("spaces_pagination").await else {
        return;
    };
    // More than the maximum page, with timestamp ties to exercise the ID tie-breaker.
    for owner in ["user-a", "user-b"] {
        sqlx::query("INSERT INTO spaces (id, owner_id, name, created_at) SELECT '00000000-0000-4000-8000-' || lpad(n::text, 12, '0'), $1, 'Space ' || n, CASE WHEN n = 101 THEN '2026-09-24T00:00:00Z'::timestamptz ELSE '2026-09-25T00:00:00Z'::timestamptz END FROM generate_series($2::int, $3::int) AS n")
            .bind(owner).bind(if owner == "user-a" {1} else {102}).bind(if owner == "user-a" {101} else {104})
            .execute(fixture.database().pool()).await.unwrap();
    }
    let default = list(&app, "user-a", "").await;
    assert_eq!(default["spaces"].as_array().unwrap().len(), 50);
    assert_eq!(default["spaces"][0]["name"], "Space 100");
    assert_eq!(default["spaces"][49]["name"], "Space 51");
    let cursor = default["nextCursor"].as_str().unwrap();
    let deleted = default["spaces"][49]["id"].as_str().unwrap();
    let response = request(
        &app,
        "DELETE",
        &format!("/v1/spaces/{deleted}"),
        Some("user-a"),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let second = list(&app, "user-a", &format!("?cursor={cursor}&limit=50")).await;
    assert_eq!(second["spaces"][0]["name"], "Space 50");
    assert_eq!(second["spaces"][49]["name"], "Space 1");
    let third = list(
        &app,
        "user-a",
        &format!("?cursor={}&limit=1", second["nextCursor"].as_str().unwrap()),
    )
    .await;
    assert_eq!(third["spaces"][0]["name"], "Space 101");
    assert_eq!(third["nextCursor"], Value::Null);
    let all = list(&app, "user-a", "?limit=100").await;
    assert_eq!(all["spaces"].as_array().unwrap().len(), 100);
    assert_eq!(all["nextCursor"], Value::Null);
    // A cursor is only a boundary: it cannot choose the owner or confer access.
    let foreign = list(&app, "user-b", "?limit=1").await;
    let from_foreign = list(
        &app,
        "user-a",
        &format!(
            "?cursor={}&limit=100",
            foreign["nextCursor"].as_str().unwrap()
        ),
    )
    .await;
    assert_eq!(from_foreign, all);
    assert_eq!(
        list(&app, "unprovisioned", &format!("?cursor={cursor}")).await,
        json!({"spaces":[],"nextCursor":null})
    );
    fixture.cleanup().await;
}

#[tokio::test]
async fn spaces_delete_is_owner_scoped_durable_and_leaves_other_rows_alone() {
    let Some((app, fixture)) = setup("spaces_delete_durability").await else {
        return;
    };
    let a = create(&app, "user-a", "Research").await;
    let b = create(&app, "user-b", "Research").await;
    sqlx::query("INSERT INTO conversations (id) VALUES ('unrelated-chat')")
        .execute(fixture.database().pool())
        .await
        .unwrap();
    sqlx::query("INSERT INTO calendar_events (id, user_id, title, all_day, start_date, starts_at, ends_at) VALUES ('unrelated-event', 'user-a', 'Event', true, '2026-09-25', '2026-09-25T00:00:00Z', '2026-09-26T00:00:00Z')").execute(fixture.database().pool()).await.unwrap();
    sqlx::query("INSERT INTO document_files (id, owner_id, display_name) VALUES ('unrelated-file', 'user-a', 'File')").execute(fixture.database().pool()).await.unwrap();
    let before = unrelated_rows(fixture.database()).await;
    let path = format!("/v1/spaces/{}", a["id"].as_str().unwrap());
    let foreign = assert_error(
        request(&app, "DELETE", &path, Some("user-b"), None).await,
        StatusCode::NOT_FOUND,
    )
    .await;
    let missing = assert_error(
        request(&app, "DELETE", "/v1/spaces/missing", Some("user-b"), None).await,
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(foreign, missing);
    assert_eq!(list(&app, "user-a", "").await["spaces"], json!([a]));
    fixture.database().close().await;
    let reopened = fixture.reconnect().await;
    let app = router(reopened.clone(), &fixture);
    assert_eq!(list(&app, "user-a", "").await["spaces"], json!([a]));
    let response = request(&app, "DELETE", &path, Some("user-a"), None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .is_empty()
    );
    assert_eq!(
        assert_error(
            request(&app, "DELETE", &path, Some("user-a"), None).await,
            StatusCode::NOT_FOUND
        )
        .await,
        missing
    );
    assert_eq!(list(&app, "user-b", "").await["spaces"], json!([b]));
    assert_eq!(unrelated_rows(&reopened).await, before);
    reopened.close().await;
    let reopened = fixture.reconnect().await;
    assert_eq!(
        list(&router(reopened.clone(), &fixture), "user-a", "").await["spaces"],
        json!([])
    );
    reopened.close().await;
    fixture.cleanup().await;
}

async fn unrelated_rows(database: &Database) -> Value {
    sqlx::query_scalar("SELECT jsonb_build_object('users', (SELECT jsonb_agg(to_jsonb(u) ORDER BY id) FROM users u), 'calendar', (SELECT jsonb_agg(to_jsonb(e) ORDER BY id) FROM calendar_events e), 'chat', (SELECT jsonb_agg(to_jsonb(c) ORDER BY id) FROM conversations c), 'documents', (SELECT jsonb_agg(to_jsonb(d) ORDER BY id) FROM document_files d))")
        .fetch_one(database.pool()).await.unwrap()
}

#[tokio::test]
async fn spaces_database_constraints_indexes_cascade_and_cli_verification() {
    let Some((_app, fixture)) = setup("spaces_constraints").await else {
        return;
    };
    let pool = fixture.database().pool();
    for (id, owner, name, description, code) in [
        ("bad-empty", "user-a", "".to_owned(), None, "23514"),
        ("bad-trim", "user-a", " padded ".into(), None, "23514"),
        ("bad-long", "user-a", "é".repeat(121), None, "23514"),
        (
            "bad-description",
            "user-a",
            "Desc".into(),
            Some("é".repeat(2001)),
            "23514",
        ),
        ("bad-owner", "unknown", "Valid".into(), None, "23503"),
    ] {
        let error = sqlx::query(
            "INSERT INTO spaces (id, owner_id, name, description) VALUES ($1,$2,$3,$4)",
        )
        .bind(id)
        .bind(owner)
        .bind(name)
        .bind(description)
        .execute(pool)
        .await
        .unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some(code)
        );
    }
    sqlx::query("INSERT INTO spaces (id, owner_id, name) VALUES ('a', 'user-a', 'Research'), ('b', 'user-b', 'research')").execute(pool).await.unwrap();
    let error =
        sqlx::query("INSERT INTO spaces (id, owner_id, name) VALUES ('c', 'user-a', 'RESEARCH')")
            .execute(pool)
            .await
            .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().constraint(),
        Some("spaces_owner_name_ci_idx")
    );
    let error =
        sqlx::query("INSERT INTO spaces (id, owner_id, name) VALUES ('a', 'user-b', 'Other')")
            .execute(pool)
            .await
            .unwrap_err();
    assert!(error.as_database_error().unwrap().is_unique_violation());
    let indexes: Vec<(String, String)> = sqlx::query_as("SELECT indexname, indexdef FROM pg_indexes WHERE schemaname = current_schema() AND tablename = 'spaces'").fetch_all(pool).await.unwrap();
    for expected in [
        "spaces_pkey",
        "spaces_owner_id_key",
        "spaces_owner_name_ci_idx",
        "spaces_owner_created_idx",
    ] {
        assert!(
            indexes.iter().any(|(name, _)| name == expected),
            "missing {expected}"
        );
    }
    assert!(
        indexes
            .iter()
            .any(|(name, definition)| name == "spaces_owner_created_idx"
                && definition.contains("owner_id, created_at DESC, id DESC"))
    );
    // The composite key can support a future same-owner membership foreign key.
    sqlx::query("CREATE TABLE space_membership_probe (owner_id text, space_id text, FOREIGN KEY (owner_id, space_id) REFERENCES spaces (owner_id, id))").execute(pool).await.unwrap();
    let error = sqlx::query("INSERT INTO space_membership_probe VALUES ('user-b', 'a')")
        .execute(pool)
        .await
        .unwrap_err();
    assert!(
        error
            .as_database_error()
            .unwrap()
            .is_foreign_key_violation()
    );
    sqlx::query("DROP TABLE space_membership_probe")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE id = 'user-a'")
        .execute(pool)
        .await
        .unwrap();
    let remaining: Vec<String> = sqlx::query_scalar("SELECT id FROM spaces ORDER BY id")
        .fetch_all(pool)
        .await
        .unwrap();
    assert_eq!(remaining, vec!["b"]);
    let output = tokio::process::Command::new(env!("CARGO_BIN_EXE_curio_db"))
        .arg("verify")
        .env("DATABASE_URL", fixture.database_url())
        .env("CURIO_DB_SCHEMA", fixture.schema())
        .output()
        .await
        .unwrap();
    assert!(
        output.status.success(),
        "curio_db verify failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    fixture.cleanup().await;
}

#[tokio::test]
async fn spaces_concurrent_duplicates_commit_exactly_once() {
    let Some((app, fixture)) = setup("spaces_concurrent_duplicates").await else {
        return;
    };
    let (first, second) = tokio::join!(
        request(
            &app,
            "POST",
            "/v1/spaces",
            Some("user-a"),
            Some(json!({"name":"Research"}))
        ),
        request(
            &app,
            "POST",
            "/v1/spaces",
            Some("user-a"),
            Some(json!({"name":" research "}))
        ),
    );
    let mut statuses = [first.status().as_u16(), second.status().as_u16()];
    statuses.sort();
    assert_eq!(statuses, [201, 409]);
    assert_eq!(
        list(&app, "user-a", "").await["spaces"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    fixture.cleanup().await;
}
