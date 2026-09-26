use super::*;

#[tokio::test]
async fn inheritance_subtree_moves_and_space_detachment_are_versioned() {
    let Some((app, fixture)) = setup("tasks_space_membership").await else {
        return;
    };
    let a = space(&app, "user-a", "A").await;
    let b = space(&app, "user-a", "B").await;
    let root = create(&app, "user-a", json!({"title":"Root","spaceId":a["id"]})).await;
    let child = create(
        &app,
        "user-a",
        json!({"title":"Child","parentId":root["id"]}),
    )
    .await;
    let grandchild = create(
        &app,
        "user-a",
        json!({"title":"Grandchild","parentId":child["id"]}),
    )
    .await;
    assert_eq!(child["spaceId"], a["id"]);
    assert_eq!(grandchild["spaceId"], a["id"]);
    assert_eq!(get(&app, "user-a", &root).await["childCount"], 1);
    api(
        &app,
        "POST",
        "/v1/tasks",
        "user-a",
        Some(json!({"title":"Wrong","parentId":root["id"],"spaceId":null})),
        422,
    )
    .await;
    let root = patch(
        &app,
        "user-a",
        &get(&app, "user-a", &root).await,
        json!({"spaceId":b["id"]}),
    )
    .await;
    let moved = get(&app, "user-a", &child).await;
    assert_eq!(moved["spaceId"], b["id"]);
    assert_eq!(moved["version"], 2);
    assert_eq!(get(&app, "user-a", &grandchild).await["spaceId"], b["id"]);
    api(
        &app,
        "PATCH",
        &task_path(&child),
        "user-a",
        Some(json!({"expectedVersion":1,"title":"Stale child"})),
        409,
    )
    .await;
    api(
        &app,
        "DELETE",
        &format!("/v1/spaces/{}", b["id"].as_str().unwrap()),
        "user-a",
        None,
        204,
    )
    .await;
    for original in [&root, &moved, &grandchild] {
        let detached = get(&app, "user-a", original).await;
        assert_eq!(detached["spaceId"], Value::Null);
        assert!(detached["version"].as_i64().unwrap() > original["version"].as_i64().unwrap());
        assert!(detached["updatedAt"].as_str().unwrap() > original["updatedAt"].as_str().unwrap());
    }
    assert_eq!(get(&app, "user-a", &child).await["parentId"], root["id"]);
    fixture.cleanup().await;
}

#[tokio::test]
async fn cycle_depth_and_concurrent_reparenting_rules() {
    let Some((app, fixture)) = setup("tasks_hierarchy").await else {
        return;
    };
    let root = create(&app, "user-a", json!({"title":"Root"})).await;
    let mut current = root.clone();
    for n in 2..=5 {
        current = create(
            &app,
            "user-a",
            json!({"title":format!("Level {n}"),"parentId":current["id"]}),
        )
        .await;
    }
    api(
        &app,
        "POST",
        "/v1/tasks",
        "user-a",
        Some(json!({"title":"Too deep","parentId":current["id"]})),
        422,
    )
    .await;
    api(
        &app,
        "PATCH",
        &task_path(&root),
        "user-a",
        Some(json!({"expectedVersion":1,"parentId":current["id"]})),
        422,
    )
    .await;
    api(
        &app,
        "PATCH",
        &task_path(&root),
        "user-a",
        Some(json!({"expectedVersion":1,"parentId":root["id"]})),
        422,
    )
    .await;
    let a = create(&app, "user-a", json!({"title":"A"})).await;
    let b = create(&app, "user-a", json!({"title":"B"})).await;
    let a = get(&app, "user-a", &a).await;
    let b = get(&app, "user-a", &b).await;
    let pa = task_path(&a);
    let pb = task_path(&b);
    let (ra, rb) = tokio::join!(
        request(
            &app,
            "PATCH",
            &pa,
            Some("user-a"),
            Some(json!({"expectedVersion":a["version"],"parentId":b["id"]}))
        ),
        request(
            &app,
            "PATCH",
            &pb,
            Some("user-a"),
            Some(json!({"expectedVersion":b["version"],"parentId":a["id"]}))
        )
    );
    let statuses = [ra.status().as_u16(), rb.status().as_u16()];
    assert_eq!(statuses.iter().filter(|&&s| s == 200).count(), 1);
    assert!(statuses.iter().any(|&s| s == 409 || s == 422));
    let ar = get(&app, "user-a", &a).await;
    let br = get(&app, "user-a", &b).await;
    assert!(!(ar["parentId"] == b["id"] && br["parentId"] == a["id"]));
    fixture.cleanup().await;
}

#[tokio::test]
async fn subtree_delete_cascades_associations_but_keeps_tags_and_unrelated_data() {
    let Some((app, fixture)) = setup("tasks_subtree_delete").await else {
        return;
    };
    let tag = tag(&app, "user-a", "Keep").await;
    let root = create(&app, "user-a", json!({"title":"Root"})).await;
    let child=create(&app,"user-a",json!({"title":"Child","parentId":root["id"],"tagIds":[tag["id"]],"links":[{"kind":"curio_note","noteId":"opaque"}]})).await;
    api(
        &app,
        "POST",
        &format!("{}/comments", task_path(&child)),
        "user-a",
        Some(json!({"body":"Comment"})),
        201,
    )
    .await;
    let other = create(&app, "user-b", json!({"title":"Unrelated"})).await;
    api(
        &app,
        "DELETE",
        &format!("{}?expectedVersion=1", task_path(&root)),
        "user-a",
        None,
        204,
    )
    .await;
    api(&app, "GET", &task_path(&child), "user-a", None, 404).await;
    assert_eq!(get(&app, "user-b", &other).await, other);
    for table in ["task_tag_assignments", "task_comments", "task_links"] {
        let count: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {table}"))
            .fetch_one(fixture.database().pool())
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
    assert_eq!(
        api(&app, "GET", "/v1/task-tags", "user-a", None, 200).await["tags"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    fixture.cleanup().await;
}

#[tokio::test]
async fn manual_positions_normalize_and_invalidate_changed_siblings() {
    let Some((app, fixture)) = setup("tasks_manual").await else {
        return;
    };
    let a = create(&app, "user-a", json!({"title":"A"})).await;
    let b = create(&app, "user-a", json!({"title":"B"})).await;
    let c = create(&app, "user-a", json!({"title":"C"})).await;
    assert_eq!(b["sortOrder"], 1);
    assert_eq!(c["sortOrder"], 2);
    let moved = patch(&app, "user-a", &c, json!({"sortOrder":0})).await;
    assert_eq!(moved["sortOrder"], 0);
    assert_eq!(moved["version"], 2);
    let rows = list(&app, "user-a", "?parentId=none&sort=manual").await;
    assert_eq!(rows["tasks"][0]["id"], c["id"]);
    assert_eq!(rows["tasks"][1]["id"], a["id"]);
    assert_eq!(rows["tasks"][2]["id"], b["id"]);
    assert_eq!(rows["tasks"][1]["version"], 2);
    let moved = patch(&app, "user-a", &moved, json!({"sortOrder":10000})).await;
    assert_eq!(moved["sortOrder"], 2);
    fixture.cleanup().await;
}

#[tokio::test]
async fn reparent_inherits_space_and_checks_entire_subtree_height() {
    let Some((app, fixture)) = setup("tasks_reparent_height").await else {
        return;
    };
    let s = space(&app, "user-a", "Destination").await;
    let root = create(&app, "user-a", json!({"title":"Root","spaceId":s["id"]})).await;
    let level2 = create(
        &app,
        "user-a",
        json!({"title":"Level 2","parentId":root["id"]}),
    )
    .await;
    let level3 = create(
        &app,
        "user-a",
        json!({"title":"Level 3","parentId":level2["id"]}),
    )
    .await;
    let branch = create(&app, "user-a", json!({"title":"Branch"})).await;
    let leaf = create(
        &app,
        "user-a",
        json!({"title":"Leaf","parentId":branch["id"]}),
    )
    .await;
    let twig = create(
        &app,
        "user-a",
        json!({"title":"Twig","parentId":leaf["id"]}),
    )
    .await;
    api(
        &app,
        "PATCH",
        &task_path(&branch),
        "user-a",
        Some(json!({"expectedVersion":1,"parentId":level3["id"]})),
        422,
    )
    .await;
    let moved = patch(&app, "user-a", &branch, json!({"parentId":level2["id"]})).await;
    assert_eq!(moved["spaceId"], s["id"]);
    assert_eq!(get(&app, "user-a", &twig).await["spaceId"], s["id"]);
    let moved = patch(
        &app,
        "user-a",
        &moved,
        json!({"parentId":null,"spaceId":null}),
    )
    .await;
    assert_eq!(moved["spaceId"], Value::Null);
    assert_eq!(get(&app, "user-a", &twig).await["spaceId"], Value::Null);
    fixture.cleanup().await;
}

#[tokio::test]
async fn space_delete_and_task_create_share_the_owner_lock() {
    let Some((app, fixture)) = setup("tasks_space_delete_race").await else {
        return;
    };
    let s = space(&app, "user-a", "Race").await;
    let path = format!("/v1/spaces/{}", s["id"].as_str().unwrap());
    let (created, deleted) = tokio::join!(
        request(
            &app,
            "POST",
            "/v1/tasks",
            Some("user-a"),
            Some(json!({"title":"Concurrent","spaceId":s["id"]}))
        ),
        request(&app, "DELETE", &path, Some("user-a"), None)
    );
    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
    assert!(matches!(created.status().as_u16(), 201 | 404));
    let tasks = list(&app, "user-a", "").await;
    for task in tasks["tasks"].as_array().unwrap() {
        assert_eq!(task["spaceId"], Value::Null);
        assert_eq!(task["version"], 2);
    }
    fixture.cleanup().await;
}
