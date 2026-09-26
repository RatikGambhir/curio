use super::*;

#[tokio::test]
async fn tags_normalize_rename_and_delete_preserving_version_safety() {
    let Some((app, fixture)) = setup("task_tags").await else {
        return;
    };
    let tag = tag(&app, "user-a", "  École  ").await;
    assert_eq!(tag["name"], "École");
    api(
        &app,
        "POST",
        "/v1/task-tags",
        "user-a",
        Some(json!({"name":"école"})),
        409,
    )
    .await;
    api(
        &app,
        "POST",
        "/v1/task-tags",
        "user-b",
        Some(json!({"name":"École"})),
        201,
    )
    .await;
    let task = create(
        &app,
        "user-a",
        json!({"title":"Tagged","tagIds":[tag["id"]]}),
    )
    .await;
    let path = format!("/v1/task-tags/{}", tag["id"].as_str().unwrap());
    let renamed = api(
        &app,
        "PATCH",
        &path,
        "user-a",
        Some(json!({"name":"School"})),
        200,
    )
    .await;
    assert_eq!(renamed["id"], tag["id"]);
    assert_eq!(renamed["createdAt"], tag["createdAt"]);
    let changed = get(&app, "user-a", &task).await;
    assert_eq!(changed["tags"][0]["name"], "School");
    assert_eq!(changed["version"], 1);
    api(&app, "DELETE", &path, "user-a", None, 204).await;
    let changed = get(&app, "user-a", &task).await;
    assert_eq!(changed["tags"], json!([]));
    assert_eq!(changed["version"], 2);
    api(
        &app,
        "PATCH",
        &task_path(&task),
        "user-a",
        Some(json!({"expectedVersion":1,"tagIds":[]})),
        409,
    )
    .await;
    api(&app, "DELETE", &path, "user-a", None, 404).await;
    fixture.cleanup().await;
}

#[tokio::test]
async fn comments_links_are_independent_paged_and_durable() {
    let Some((app, fixture)) = setup("tasks_subresources").await else {
        return;
    };
    let tag = tag(&app, "user-a", "Persist").await;
    let task=create(&app,"user-a",json!({"title":"Task","tagIds":[tag["id"]],"links":[{"kind":"web","url":"https://example.com/brief","label":" Brief "},{"kind":"curio_note","noteId":"unresolved-other-owner-note"}]})).await;
    assert_eq!(task["links"].as_array().unwrap().len(), 2);
    assert_eq!(task["links"][0]["label"], "Brief");
    assert_eq!(task["links"][1]["noteId"], "unresolved-other-owner-note");
    let cp = format!("{}/comments", task_path(&task));
    let first = api(
        &app,
        "POST",
        &cp,
        "user-a",
        Some(json!({"body":" First comment "})),
        201,
    )
    .await;
    let second = api(
        &app,
        "POST",
        &cp,
        "user-a",
        Some(json!({"body":"Second\nline"})),
        201,
    )
    .await;
    assert_eq!(first["authorId"], "user-a");
    assert_eq!(first["body"], "First comment");
    assert_eq!(first["editedAt"], Value::Null);
    let page = api(&app, "GET", &format!("{cp}?limit=1"), "user-a", None, 200).await;
    let next = api(
        &app,
        "GET",
        &format!(
            "{cp}?limit=1&cursor={}",
            page["nextCursor"].as_str().unwrap()
        ),
        "user-a",
        None,
        200,
    )
    .await;
    assert_ne!(page["comments"][0]["id"], next["comments"][0]["id"]);
    assert_eq!(next["nextCursor"], Value::Null);
    let edited = api(
        &app,
        "PATCH",
        &format!("{cp}/{}", first["id"].as_str().unwrap()),
        "user-a",
        Some(json!({"body":"Edited"})),
        200,
    )
    .await;
    assert_eq!(edited["createdAt"], first["createdAt"]);
    assert!(edited["editedAt"].is_string());
    let lp = format!("{}/links", task_path(&task));
    let added = api(
        &app,
        "POST",
        &lp,
        "user-a",
        Some(json!({"kind":"web","url":"http://example.com/added","sortOrder":0})),
        201,
    )
    .await;
    assert_eq!(added["sortOrder"], 0);
    let edited_link = api(
        &app,
        "PATCH",
        &format!("{lp}/{}", added["id"].as_str().unwrap()),
        "user-a",
        Some(json!({"kind":"curio_note","url":null,"noteId":"opaque","label":" Note "})),
        200,
    )
    .await;
    assert_eq!(edited_link["url"], Value::Null);
    assert_eq!(edited_link["noteId"], "opaque");
    assert_eq!(edited_link["createdAt"], added["createdAt"]);
    let detail = get(&app, "user-a", &task).await;
    assert_eq!(detail["version"], 1);
    assert_eq!(detail["updatedAt"], task["updatedAt"]);
    assert_eq!(detail["commentCount"], 2);
    fixture.database().close().await;
    let reopened = fixture.reconnect().await;
    let app = router(reopened.clone(), &fixture);
    assert_eq!(get(&app, "user-a", &task).await, detail);
    assert_eq!(
        api(&app, "GET", &format!("{cp}?limit=100"), "user-a", None, 200).await["comments"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    api(
        &app,
        "DELETE",
        &format!("{cp}/{}", second["id"].as_str().unwrap()),
        "user-a",
        None,
        204,
    )
    .await;
    api(
        &app,
        "DELETE",
        &format!("{lp}/{}", added["id"].as_str().unwrap()),
        "user-a",
        None,
        204,
    )
    .await;
    let detail = get(&app, "user-a", &task).await;
    assert_eq!(detail["commentCount"], 1);
    assert_eq!(detail["version"], 1);
    assert_eq!(detail["links"].as_array().unwrap().len(), 2);
    let changed = patch(&app, "user-a", &task, json!({"title":"Still current"})).await;
    assert_eq!(changed["version"], 2);
    reopened.close().await;
    fixture.cleanup().await;
}

#[tokio::test]
async fn bounded_links_and_failed_batches_are_atomic() {
    let Some((app, fixture)) = setup("tasks_link_limits").await else {
        return;
    };
    let links: Vec<_> = (0..50)
        .map(|n| json!({"kind":"curio_note","noteId":format!("note-{n}")}))
        .collect();
    let task = create(&app, "user-a", json!({"title":"Fifty","links":links})).await;
    api(
        &app,
        "POST",
        &format!("{}/links", task_path(&task)),
        "user-a",
        Some(json!({"kind":"web","url":"https://example.com"})),
        422,
    )
    .await;
    assert_eq!(
        get(&app, "user-a", &task).await["links"]
            .as_array()
            .unwrap()
            .len(),
        50
    );
    api(&app,"POST","/v1/tasks","user-a",Some(json!({"title":"Invalid aggregate","links":[{"kind":"web","url":"https://example.com"},{"kind":"web","url":"file:///secret"}]})),422).await;
    assert_eq!(
        list(&app, "user-a", "").await["tasks"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let link = &task["links"][0];
    api(
        &app,
        "PATCH",
        &format!(
            "{}/links/{}",
            task_path(&task),
            link["id"].as_str().unwrap()
        ),
        "user-a",
        Some(json!({"kind":"web","url":"https://example.com"})),
        422,
    )
    .await;
    assert_eq!(get(&app, "user-a", &task).await["links"][0], *link);
    fixture.cleanup().await;
}
