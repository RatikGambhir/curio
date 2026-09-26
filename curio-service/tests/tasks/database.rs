use super::*;

#[tokio::test]
async fn composite_keys_constraints_indexes_and_catalog_are_enforced() {
    let Some((app, fixture)) = setup("tasks_database").await else {
        return;
    };
    let pool = fixture.database().pool();
    let space = space(&app, "user-a", "Owned").await;
    let tag = tag(&app, "user-a", "Owned").await;
    let a = create(&app, "user-a", json!({"title":"A"})).await;
    let b = create(&app, "user-b", json!({"title":"B"})).await;
    let aid = a["id"].as_str().unwrap();
    let bid = b["id"].as_str().unwrap();
    let statements = [
        (
            "UPDATE tasks SET space_id=$2 WHERE id=$1",
            bid,
            space["id"].as_str().unwrap(),
        ),
        ("UPDATE tasks SET parent_id=$2 WHERE id=$1", bid, aid),
        (
            "INSERT INTO task_tag_assignments(owner_id,task_id,tag_id) VALUES ('user-b',$1,$2)",
            bid,
            tag["id"].as_str().unwrap(),
        ),
        (
            "INSERT INTO task_comments(id,owner_id,task_id,author_id,body) VALUES ($1,'user-b',$2,'user-b','Comment')",
            "bad-comment",
            aid,
        ),
        (
            "INSERT INTO task_links(id,owner_id,task_id,kind,note_id) VALUES ($1,'user-b',$2,'curio_note','opaque')",
            "bad-link",
            aid,
        ),
    ];
    for (sql, one, two) in statements {
        let error = sqlx::query(sql)
            .bind(one)
            .bind(two)
            .execute(pool)
            .await
            .unwrap_err();
        assert!(
            error
                .as_database_error()
                .unwrap()
                .is_foreign_key_violation()
        );
    }
    for clause in [
        "title=''",
        "status='unknown'",
        "priority='urgent'",
        "version=0",
        "sort_order=-1",
        "parent_id=id",
        "due_on='2026-01-01',due_at='2026-01-01T12:00:00Z',due_timezone='UTC'",
        "due_at='2026-01-01T12:00:00Z'",
        "status='done'",
        "completed_at=now()",
    ] {
        let error = sqlx::query(&format!("UPDATE tasks SET {clause} WHERE id=$1"))
            .bind(aid)
            .execute(pool)
            .await
            .unwrap_err();
        assert_eq!(
            error.as_database_error().unwrap().code().as_deref(),
            Some("23514"),
            "{clause}"
        );
    }
    let indexes: Vec<String> =
        sqlx::query_scalar("SELECT indexname FROM pg_indexes WHERE schemaname=current_schema()")
            .fetch_all(pool)
            .await
            .unwrap();
    for name in [
        "tasks_owner_created_idx",
        "tasks_space_order_idx",
        "tasks_parent_order_idx",
        "tasks_due_on_idx",
        "tasks_due_at_idx",
        "tasks_status_created_idx",
        "tasks_flagged_created_idx",
        "task_tag_assignments_tag_idx",
        "task_comments_task_created_idx",
        "task_links_task_order_idx",
    ] {
        assert!(indexes.contains(&name.to_owned()), "{name}");
    }
    // Prove selective date-only filtering can use the partial due index.
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query("SET enable_seqscan=off")
        .execute(&mut *conn)
        .await
        .unwrap();
    let plan:Vec<String>=sqlx::query_scalar("EXPLAIN SELECT id FROM tasks WHERE owner_id='user-a' AND due_on >= '2026-01-01' AND due_on < '2026-02-01' ORDER BY due_on,id LIMIT 50").fetch_all(&mut *conn).await.unwrap();
    assert!(plan.join(" ").contains("tasks_due_on_idx"));
    sqlx::query("RESET enable_seqscan")
        .execute(&mut *conn)
        .await
        .unwrap();
    drop(conn);
    let output = tokio::process::Command::new(env!("CARGO_BIN_EXE_curio_db"))
        .arg("verify")
        .env("DATABASE_URL", fixture.database_url())
        .env("CURIO_DB_SCHEMA", fixture.schema())
        .output()
        .await
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    fixture.cleanup().await;
}

#[tokio::test]
async fn injected_storage_failure_rolls_back_create_and_tag_replacement() {
    let Some((app, fixture)) = setup("tasks_rollback").await else {
        return;
    };
    let pool = fixture.database().pool();
    let a = tag(&app, "user-a", "A").await;
    let b = tag(&app, "user-a", "B").await;
    sqlx::raw_sql("CREATE FUNCTION reject_test_link() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.note_id='fail-write' THEN RAISE EXCEPTION 'private injected failure'; END IF; RETURN NEW; END $$; CREATE TRIGGER reject_test_link BEFORE INSERT ON task_links FOR EACH ROW EXECUTE FUNCTION reject_test_link();").execute(pool).await.unwrap();
    let error=api(&app,"POST","/v1/tasks","user-a",Some(json!({"title":"Must roll back","tagIds":[a["id"]],"links":[{"kind":"curio_note","noteId":"first"},{"kind":"curio_note","noteId":"fail-write"}]})),500).await;
    assert_eq!(error, json!({"error":"Tasks are unavailable."}));
    for table in ["tasks", "task_tag_assignments", "task_links"] {
        assert_eq!(
            sqlx::query_scalar::<_, i64>(&format!("SELECT count(*) FROM {table}"))
                .fetch_one(pool)
                .await
                .unwrap(),
            0
        );
    }
    let task = create(
        &app,
        "user-a",
        json!({"title":"Original","tagIds":[a["id"]]}),
    )
    .await;
    sqlx::raw_sql("CREATE FUNCTION reject_test_assignment() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF EXISTS(SELECT 1 FROM task_tags WHERE id=NEW.tag_id AND name='B') THEN RAISE EXCEPTION 'private injected failure'; END IF; RETURN NEW; END $$; CREATE TRIGGER reject_test_assignment BEFORE INSERT ON task_tag_assignments FOR EACH ROW EXECUTE FUNCTION reject_test_assignment();").execute(pool).await.unwrap();
    api(
        &app,
        "PATCH",
        &task_path(&task),
        "user-a",
        Some(json!({"expectedVersion":1,"title":"Partial change","tagIds":[b["id"]]})),
        500,
    )
    .await;
    assert_eq!(get(&app, "user-a", &task).await, task);
    fixture.cleanup().await;
}

#[tokio::test]
async fn deleting_a_user_cascades_all_task_data_without_cross_owner_changes() {
    let Some((app, fixture)) = setup("tasks_user_cascade").await else {
        return;
    };
    let s = space(&app, "user-a", "Space").await;
    let a = tag(&app, "user-a", "A").await;
    let root = create(
        &app,
        "user-a",
        json!({"title":"Root","spaceId":s["id"],"tagIds":[a["id"]]}),
    )
    .await;
    let child=create(&app,"user-a",json!({"title":"Child","parentId":root["id"],"links":[{"kind":"curio_note","noteId":"opaque"}]})).await;
    api(
        &app,
        "POST",
        &format!("{}/comments", task_path(&child)),
        "user-a",
        Some(json!({"body":"Comment"})),
        201,
    )
    .await;
    let foreign = create(&app, "user-b", json!({"title":"Unchanged"})).await;
    sqlx::query("DELETE FROM users WHERE id='user-a'")
        .execute(fixture.database().pool())
        .await
        .unwrap();
    assert_eq!(get(&app, "user-b", &foreign).await, foreign);
    for table in [
        "tasks",
        "task_tags",
        "task_tag_assignments",
        "task_comments",
        "task_links",
    ] {
        assert_eq!(
            sqlx::query_scalar::<_, i64>(&format!(
                "SELECT count(*) FROM {table} WHERE owner_id='user-a'"
            ))
            .fetch_one(fixture.database().pool())
            .await
            .unwrap(),
            0
        );
    }
    fixture.cleanup().await;
}
