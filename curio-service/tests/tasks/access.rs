use super::*;

#[tokio::test]
async fn every_route_requires_bearer_and_never_accepts_an_owner_field() {
    let Some((app, fixture)) = setup("tasks_auth").await else {
        return;
    };
    for (method, path) in [
        ("POST", "/v1/tasks"),
        ("GET", "/v1/tasks"),
        ("GET", "/v1/tasks/id"),
        ("PATCH", "/v1/tasks/id"),
        ("DELETE", "/v1/tasks/id?expectedVersion=1"),
        ("POST", "/v1/task-tags"),
        ("GET", "/v1/task-tags"),
        ("PATCH", "/v1/task-tags/id"),
        ("DELETE", "/v1/task-tags/id"),
        ("POST", "/v1/tasks/id/comments"),
        ("GET", "/v1/tasks/id/comments"),
        ("PATCH", "/v1/tasks/id/comments/comment"),
        ("DELETE", "/v1/tasks/id/comments/comment"),
        ("POST", "/v1/tasks/id/links"),
        ("PATCH", "/v1/tasks/id/links/link"),
        ("DELETE", "/v1/tasks/id/links/link"),
    ] {
        for owner in [None, Some(""), Some("not valid")] {
            let response = request(&app, method, path, owner, None).await;
            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "{method} {path}"
            );
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
    for field in ["ownerId", "userId", "id", "createdAt", "completedAt"] {
        let mut body = json!({"title":"Task"});
        body[field] = json!("forged");
        api(&app, "POST", "/v1/tasks", "user-a", Some(body), 422).await;
    }
    for path in [
        "/v1/tasks?ownerId=user-b",
        "/v1/tasks?userId=user-b",
        "/v1/task-tags?ownerId=user-b",
    ] {
        api(&app, "GET", path, "user-a", None, 400).await;
    }
    let task = create(&app, "user-a", json!({"title":"Task"})).await;
    for (method, path, body) in [
        (
            "PATCH",
            task_path(&task),
            json!({"expectedVersion":1,"ownerId":"user-b"}),
        ),
        (
            "POST",
            "/v1/task-tags".into(),
            json!({"name":"Tag","ownerId":"user-b"}),
        ),
        (
            "POST",
            format!("{}/comments", task_path(&task)),
            json!({"body":"Comment","authorId":"user-b"}),
        ),
        (
            "POST",
            format!("{}/links", task_path(&task)),
            json!({"kind":"web","url":"https://example.com","ownerId":"user-b"}),
        ),
    ] {
        api(&app, method, &path, "user-a", Some(body), 422).await;
    }
    assert_eq!(
        list(&app, "unknown", "").await,
        json!({"tasks":[],"nextCursor":null})
    );
    api(
        &app,
        "POST",
        "/v1/tasks",
        "unknown",
        Some(json!({"title":"Task"})),
        422,
    )
    .await;
    api(
        &app,
        "POST",
        "/v1/task-tags",
        "unknown",
        Some(json!({"name":"Tag"})),
        422,
    )
    .await;
    fixture.cleanup().await;
}

#[tokio::test]
async fn foreign_tasks_tags_comments_links_and_references_are_not_enumerable() {
    let Some((app, fixture)) = setup("tasks_ownership").await else {
        return;
    };
    let s = space(&app, "user-a", "Space").await;
    let tag = tag(&app, "user-a", "Tag").await;
    let task=create(&app,"user-a",json!({"title":"Private","spaceId":s["id"],"tagIds":[tag["id"]],"links":[{"kind":"curio_note","noteId":"arbitrary-note"}]})).await;
    let comment = api(
        &app,
        "POST",
        &format!("{}/comments", task_path(&task)),
        "user-a",
        Some(json!({"body":"Private"})),
        201,
    )
    .await;
    let link = task["links"][0].clone();
    let missing = api(
        &app,
        "GET",
        &format!("/v1/tasks/{}", uuid()),
        "user-b",
        None,
        404,
    )
    .await;
    for (method, path, body) in [
        ("GET", task_path(&task), None),
        (
            "PATCH",
            task_path(&task),
            Some(json!({"expectedVersion":1,"title":"Forged"})),
        ),
        (
            "DELETE",
            format!("{}?expectedVersion=1", task_path(&task)),
            None,
        ),
        ("GET", format!("{}/comments", task_path(&task)), None),
        (
            "POST",
            format!("{}/comments", task_path(&task)),
            Some(json!({"body":"Forged"})),
        ),
        (
            "PATCH",
            format!(
                "{}/comments/{}",
                task_path(&task),
                comment["id"].as_str().unwrap()
            ),
            Some(json!({"body":"Forged"})),
        ),
        (
            "DELETE",
            format!(
                "{}/comments/{}",
                task_path(&task),
                comment["id"].as_str().unwrap()
            ),
            None,
        ),
        (
            "POST",
            format!("{}/links", task_path(&task)),
            Some(json!({"kind":"web","url":"https://example.com"})),
        ),
        (
            "PATCH",
            format!(
                "{}/links/{}",
                task_path(&task),
                link["id"].as_str().unwrap()
            ),
            Some(json!({"label":"Forged"})),
        ),
        (
            "DELETE",
            format!(
                "{}/links/{}",
                task_path(&task),
                link["id"].as_str().unwrap()
            ),
            None,
        ),
        (
            "PATCH",
            format!("/v1/task-tags/{}", tag["id"].as_str().unwrap()),
            Some(json!({"name":"Forged"})),
        ),
        (
            "DELETE",
            format!("/v1/task-tags/{}", tag["id"].as_str().unwrap()),
            None,
        ),
    ] {
        assert_eq!(api(&app, method, &path, "user-b", body, 404).await, missing);
    }
    for body in [
        json!({"title":"Foreign space","spaceId":s["id"]}),
        json!({"title":"Foreign parent","parentId":task["id"]}),
        json!({"title":"Foreign tag","tagIds":[tag["id"]]}),
    ] {
        assert_eq!(
            api(&app, "POST", "/v1/tasks", "user-b", Some(body), 404).await,
            missing
        );
    }
    let own = create(&app, "user-b", json!({"title":"Owned"})).await;
    for body in [
        json!({"expectedVersion":1,"spaceId":s["id"]}),
        json!({"expectedVersion":1,"parentId":task["id"]}),
        json!({"expectedVersion":1,"tagIds":[tag["id"]]}),
    ] {
        api(&app, "PATCH", &task_path(&own), "user-b", Some(body), 404).await;
    }
    // Even another task owned by the same user cannot address the wrong task's children.
    let own_a = create(&app, "user-a", json!({"title":"Other"})).await;
    api(
        &app,
        "DELETE",
        &format!(
            "{}/links/{}",
            task_path(&own_a),
            link["id"].as_str().unwrap()
        ),
        "user-a",
        None,
        404,
    )
    .await;
    api(
        &app,
        "DELETE",
        &format!(
            "{}/comments/{}",
            task_path(&own_a),
            comment["id"].as_str().unwrap()
        ),
        "user-a",
        None,
        404,
    )
    .await;
    assert_eq!(get(&app, "user-a", &task).await["title"], "Private");
    assert_eq!(get(&app, "user-b", &own).await, own);
    fixture.cleanup().await;
}

#[tokio::test]
async fn malformed_invalid_and_oversized_inputs_have_sanitized_errors() {
    let Some((app, fixture)) = setup("tasks_invalid").await else {
        return;
    };
    for body in [
        json!({"title":""}),
        json!({"title":"x".repeat(241)}),
        json!({"title":"x","description":"x".repeat(10001)}),
        json!({"title":"x","status":"unknown"}),
        json!({"title":"x","priority":"urgent"}),
        json!({"title":"x","dueOn":"2026-02-30"}),
        json!({"title":"x","dueAt":"2026-01-01T00:00:00Z"}),
        json!({"title":"x","dueTimezone":"UTC"}),
        json!({"title":"x","dueAt":"2026-01-01T00:00:00Z","dueTimezone":"Unknown/Timezone"}),
        json!({"title":"x","tagIds":vec![uuid();21]}),
        json!({"title":"x","links":vec![json!({"kind":"curio_note","noteId":"n"});51]}),
    ] {
        let error = api(&app, "POST", "/v1/tasks", "user-a", Some(body), 422).await;
        assert_eq!(error.as_object().unwrap().len(), 1);
    }
    let task = create(&app, "user-a", json!({"title":"Valid"})).await;
    for body in [
        json!({"title":"No version"}),
        json!({"expectedVersion":1}),
        json!({"expectedVersion":0,"title":"Invalid version"}),
        json!({"expectedVersion":1,"title":null}),
        json!({"expectedVersion":1,"flagged":null}),
        json!({"expectedVersion":1,"tagIds":null}),
        json!({"expectedVersion":1,"sortOrder":-1}),
    ] {
        api(&app, "PATCH", &task_path(&task), "user-a", Some(body), 422).await;
    }
    for query in [
        "",
        "?expectedVersion=no",
        "?expectedVersion=1&expectedVersion=1",
    ] {
        api(
            &app,
            "DELETE",
            &format!("{}{query}", task_path(&task)),
            "user-a",
            None,
            400,
        )
        .await;
    }
    for query in [
        "?statuses=todo,",
        "?limit=0",
        "?limit=1&limit=2",
        "?flagged=no",
        "?tagIds=",
        "?tagMode=any",
        "?sort=due",
        "?q=search",
        "?cursor=bad",
        "?dueKind=timed&dueFrom=2026-01-01",
    ] {
        api(
            &app,
            "GET",
            &format!("/v1/tasks{query}"),
            "user-a",
            None,
            400,
        )
        .await;
    }
    let malformed = raw_request(
        &app,
        "POST",
        "/v1/tasks",
        Some("user-a"),
        Some("{".into()),
        true,
    )
    .await;
    assert_eq!(malformed.status(), StatusCode::BAD_REQUEST);
    assert!(json_body(malformed).await["error"].is_string());
    let missing_type = raw_request(
        &app,
        "POST",
        "/v1/tasks",
        Some("user-a"),
        Some("{}".into()),
        false,
    )
    .await;
    assert_eq!(missing_type.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let oversized = request(
        &app,
        "POST",
        "/v1/tasks",
        Some("user-a"),
        Some(json!({"title":"x".repeat(1024*1024)})),
    )
    .await;
    assert_eq!(oversized.status(), StatusCode::PAYLOAD_TOO_LARGE);
    fixture.database().close().await;
    assert_eq!(
        api(&app, "GET", "/v1/tasks", "user-a", None, 500).await,
        json!({"error":"Tasks are unavailable."})
    );
    fixture.cleanup().await;
}
