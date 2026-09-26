use super::*;

#[tokio::test]
async fn composable_filters_keep_dates_instants_and_statuses_distinct() {
    let Some((app, fixture)) = setup("tasks_filters").await else {
        return;
    };
    let s = space(&app, "user-a", "Space").await;
    let a = tag(&app, "user-a", "A").await;
    let b = tag(&app, "user-a", "B").await;
    let date=create(&app,"user-a",json!({"title":"Date","spaceId":s["id"],"dueOn":"2026-10-02","flagged":true,"priority":"high","tagIds":[a["id"],b["id"]]})).await;
    let timed=create(&app,"user-a",json!({"title":"Timed","dueAt":"2026-10-02T12:00:00-05:00","dueTimezone":"America/Chicago","priority":"low","tagIds":[a["id"]]})).await;
    let done = create(&app, "user-a", json!({"title":"Done","status":"done"})).await;
    create(
        &app,
        "user-a",
        json!({"title":"Cancelled","status":"cancelled"}),
    )
    .await;
    create(
        &app,
        "user-b",
        json!({"title":"Foreign","dueOn":"2026-10-02"}),
    )
    .await;
    assert_eq!(
        list(&app, "user-a", "").await["tasks"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        list(&app, "user-a", "?statuses=all").await["tasks"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    for (query,id) in [
        ("?dueKind=date&dueFrom=2026-10-02&dueBefore=2026-10-03&sort=due".into(),date["id"].clone()),
        ("?dueKind=timed&dueFrom=2026-10-02T17:00:00Z&dueBefore=2026-10-02T18:00:00Z&sort=due".into(),timed["id"].clone()),
        ("?priority=high&flagged=true".into(),date["id"].clone()),
        (format!("?spaceId={}",s["id"].as_str().unwrap()),date["id"].clone()),
        ("?spaceId=none".into(),timed["id"].clone()),
        (format!("?tagIds={},{}&tagMode=all",a["id"].as_str().unwrap(),b["id"].as_str().unwrap()),date["id"].clone()),
        ("?statuses=done&completedFrom=2020-01-01T00:00:00Z&completedBefore=9999-01-01T00:00:00Z".into(),done["id"].clone()),
    ] {
        let rows=list(&app,"user-a",&query).await;assert_eq!(rows["tasks"].as_array().unwrap().len(),1,"{query}");assert_eq!(rows["tasks"][0]["id"],id);
    }
    assert_eq!(
        list(&app, "user-a", "?dueKind=date&dueBefore=2026-10-02").await["tasks"],
        json!([])
    );
    assert_eq!(
        list(
            &app,
            "user-a",
            "?dueKind=timed&dueBefore=2026-10-02T17:00:00Z"
        )
        .await["tasks"],
        json!([])
    );
    assert_eq!(
        list(
            &app,
            "user-a",
            &format!(
                "?tagIds={},{}&tagMode=any",
                a["id"].as_str().unwrap(),
                b["id"].as_str().unwrap()
            )
        )
        .await["tasks"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        list(&app, "user-a", "?hasDueDate=false&statuses=all").await["tasks"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    fixture.cleanup().await;
}

#[tokio::test]
async fn every_sort_pages_ties_and_rejects_cursor_filter_changes() {
    let Some((app, fixture)) = setup("tasks_pagination").await else {
        return;
    };
    for n in 0..4 {
        create(
            &app,
            "user-a",
            json!({"title":format!("Task {n}"),"dueOn":"2026-10-02","priority":"medium"}),
        )
        .await;
    }
    sqlx::query("UPDATE tasks SET created_at='2026-09-25T00:00:00Z' WHERE owner_id='user-a'")
        .execute(fixture.database().pool())
        .await
        .unwrap();
    for query in [
        "sort=created",
        "sort=priority",
        "sort=manual&parentId=none",
        "sort=due&dueKind=date",
    ] {
        let all = list(&app, "user-a", &format!("?{query}")).await;
        let mut ids = Vec::new();
        let mut cursor = None::<String>;
        loop {
            let page = list(
                &app,
                "user-a",
                &format!(
                    "?{query}&limit=1{}",
                    cursor
                        .as_ref()
                        .map(|c| format!("&cursor={c}"))
                        .unwrap_or_default()
                ),
            )
            .await;
            ids.push(page["tasks"][0]["id"].clone());
            if page["nextCursor"].is_null() {
                break;
            }
            cursor = Some(page["nextCursor"].as_str().unwrap().into());
            assert!(ids.len() < 10);
        }
        assert_eq!(
            ids,
            all["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|t| t["id"].clone())
                .collect::<Vec<_>>()
        );
    }
    let page = list(&app, "user-a", "?limit=2").await;
    let cursor = page["nextCursor"].as_str().unwrap();
    api(
        &app,
        "GET",
        &format!("/v1/tasks?cursor={cursor}&flagged=true"),
        "user-a",
        None,
        400,
    )
    .await;
    let boundary = &page["tasks"][1];
    api(
        &app,
        "DELETE",
        &format!(
            "{}?expectedVersion={}",
            task_path(boundary),
            boundary["version"]
        ),
        "user-a",
        None,
        204,
    )
    .await;
    let next = list(&app, "user-a", &format!("?cursor={cursor}&limit=100")).await;
    assert_eq!(next["tasks"].as_array().unwrap().len(), 2);
    assert_eq!(
        list(&app, "user-b", &format!("?cursor={cursor}")).await["tasks"],
        json!([])
    );
    fixture.cleanup().await;
}

#[tokio::test]
async fn task_tag_and_comment_pages_are_bounded_and_scoped() {
    let Some((app, fixture)) = setup("tasks_page_bounds").await else {
        return;
    };
    sqlx::query("INSERT INTO tasks (id,owner_id,title,sort_order) SELECT '00000000-0000-4000-8000-'||lpad(n::text,12,'0'),'user-a','Task '||n,n FROM generate_series(1,101) n").execute(fixture.database().pool()).await.unwrap();
    assert_eq!(
        list(&app, "user-a", "").await["tasks"]
            .as_array()
            .unwrap()
            .len(),
        50
    );
    let page = list(&app, "user-a", "?limit=100").await;
    assert_eq!(page["tasks"].as_array().unwrap().len(), 100);
    assert!(page["nextCursor"].is_string());
    for n in 0..3 {
        tag(&app, "user-a", &format!("Tag {n}")).await;
    }
    let first = api(&app, "GET", "/v1/task-tags?limit=2", "user-a", None, 200).await;
    let second = api(
        &app,
        "GET",
        &format!(
            "/v1/task-tags?cursor={}&limit=2",
            first["nextCursor"].as_str().unwrap()
        ),
        "user-a",
        None,
        200,
    )
    .await;
    assert_eq!(second["tags"].as_array().unwrap().len(), 1);
    assert_eq!(second["nextCursor"], Value::Null);
    let id = page["tasks"][0]["id"].as_str().unwrap();
    api(
        &app,
        "GET",
        &format!(
            "/v1/tasks/{id}/comments?cursor={}",
            first["nextCursor"].as_str().unwrap()
        ),
        "user-a",
        None,
        400,
    )
    .await;
    fixture.cleanup().await;
}

#[tokio::test]
async fn timed_due_and_priority_order_have_correct_cursor_keys() {
    let Some((app, fixture)) = setup("tasks_sort_values").await else {
        return;
    };
    for (name, priority, time) in [
        ("Low", Some("low"), "2026-10-02T09:00:00Z"),
        ("High", Some("high"), "2026-10-02T10:00:00Z"),
        ("None", None, "2026-10-02T11:00:00Z"),
        ("Medium", Some("medium"), "2026-10-02T12:00:00Z"),
    ] {
        create(
            &app,
            "user-a",
            json!({"title":name,"priority":priority,"dueAt":time,"dueTimezone":"UTC"}),
        )
        .await;
    }
    for (query, names) in [
        ("sort=priority", vec!["High", "Medium", "Low", "None"]),
        (
            "sort=due&dueKind=timed",
            vec!["Low", "High", "None", "Medium"],
        ),
    ] {
        let mut seen = Vec::new();
        let mut cursor = None::<String>;
        loop {
            let page = list(
                &app,
                "user-a",
                &format!(
                    "?{query}&limit=1{}",
                    cursor
                        .as_ref()
                        .map(|c| format!("&cursor={c}"))
                        .unwrap_or_default()
                ),
            )
            .await;
            seen.push(page["tasks"][0]["title"].as_str().unwrap().to_owned());
            if page["nextCursor"].is_null() {
                break;
            }
            cursor = Some(page["nextCursor"].as_str().unwrap().into());
            assert!(seen.len() < 10);
        }
        assert_eq!(seen, names);
    }
    for query in [
        format!("spaceId={}", uuid()),
        format!("parentId={}", uuid()),
        format!("tagIds={}", uuid()),
    ] {
        api(
            &app,
            "GET",
            &format!("/v1/tasks?{query}"),
            "user-a",
            None,
            404,
        )
        .await;
    }
    fixture.cleanup().await;
}
