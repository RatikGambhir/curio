use super::*;

#[tokio::test]
async fn task_round_trip_completion_reopening_and_null_patches() {
    let Some((app, fixture)) = setup("tasks_core").await else {
        return;
    };
    let task=create(&app,"user-a",json!({"title":"  Review  ","description":"  Details\nMore  ","dueOn":"2026-10-02","priority":"high","flagged":true})).await;
    assert_eq!(task["title"], "Review");
    assert_eq!(task["description"], "Details\nMore");
    assert_eq!(task["dueOn"], "2026-10-02");
    assert_eq!(task["dueAt"], Value::Null);
    assert_eq!(task["spaceId"], Value::Null);
    assert_eq!(task["status"], "todo");
    assert_eq!(task["version"], 1);
    assert_eq!(task["createdAt"], task["updatedAt"]);
    assert!(task.get("ownerId").is_none());
    assert!(task.get("userId").is_none());
    assert!(uuid::Uuid::parse_str(task["id"].as_str().unwrap()).is_ok());
    assert_eq!(task["tags"], json!([]));
    assert_eq!(task["links"], json!([]));
    assert_eq!(task["commentCount"], 0);
    assert_eq!(task["childCount"], 0);
    assert_eq!(get(&app, "user-a", &task).await, task);
    let done = patch(&app, "user-a", &task, json!({"status":"done"})).await;
    assert_eq!(done["version"], 2);
    assert_eq!(done["createdAt"], task["createdAt"]);
    assert!(done["updatedAt"].as_str().unwrap() > task["updatedAt"].as_str().unwrap());
    assert_eq!(done["completedAt"].as_str().unwrap().len(), 24);
    assert_eq!(done["dueOn"], task["dueOn"]);
    let edited = patch(
        &app,
        "user-a",
        &done,
        json!({"description":null,"priority":null,"title":"Reviewed"}),
    )
    .await;
    assert_eq!(edited["completedAt"], done["completedAt"]);
    assert_eq!(edited["description"], Value::Null);
    assert_eq!(edited["priority"], Value::Null);
    let reopened = patch(&app, "user-a", &edited, json!({"status":"in_progress"})).await;
    assert_eq!(reopened["completedAt"], Value::Null);
    let switched = patch(
        &app,
        "user-a",
        &reopened,
        json!({"dueOn":null,"dueAt":"2026-10-02T12:30:00-05:00","dueTimezone":"America/Chicago"}),
    )
    .await;
    assert_eq!(switched["dueOn"], Value::Null);
    assert_eq!(switched["dueAt"], "2026-10-02T17:30:00.000Z");
    let cleared = patch(
        &app,
        "user-a",
        &switched,
        json!({"dueAt":null,"dueTimezone":null,"flagged":false}),
    )
    .await;
    assert_eq!(cleared["dueAt"], Value::Null);
    assert_eq!(cleared["dueTimezone"], Value::Null);
    let born_done = create(
        &app,
        "user-a",
        json!({"title":"Already done","status":"done"}),
    )
    .await;
    assert!(born_done["completedAt"].is_string());
    fixture.cleanup().await;
}

#[tokio::test]
async fn stale_versions_and_invalid_aggregates_never_partly_write() {
    let Some((app, fixture)) = setup("tasks_versions").await else {
        return;
    };
    let t1 = tag(&app, "user-a", "One").await;
    let t2 = tag(&app, "user-a", "Two").await;
    let task = create(
        &app,
        "user-a",
        json!({"title":"Original","tagIds":[t1["id"]],"dueOn":"2026-10-01"}),
    )
    .await;
    let changed = patch(&app, "user-a", &task, json!({"title":"Changed"})).await;
    api(
        &app,
        "PATCH",
        &task_path(&task),
        "user-a",
        Some(json!({"expectedVersion":1,"title":"Stale","tagIds":[t2["id"]]})),
        409,
    )
    .await;
    api(
        &app,
        "DELETE",
        &format!("{}?expectedVersion=1", task_path(&task)),
        "user-a",
        None,
        409,
    )
    .await;
    assert_eq!(get(&app, "user-a", &task).await, changed);
    for body in [
        json!({"expectedVersion":2,"title":"Partial","tagIds":[t2["id"],uuid()]}),
        json!({"expectedVersion":2,"dueAt":"2026-10-01T12:00:00Z","dueTimezone":"UTC"}),
    ] {
        let expected = if body.get("tagIds").is_some() {
            404
        } else {
            422
        };
        api(
            &app,
            "PATCH",
            &task_path(&task),
            "user-a",
            Some(body),
            expected,
        )
        .await;
        assert_eq!(get(&app, "user-a", &task).await, changed);
    }
    let cleared = patch(&app, "user-a", &changed, json!({"tagIds":[]})).await;
    assert_eq!(cleared["tags"], json!([]));
    api(
        &app,
        "DELETE",
        &format!(
            "{}?expectedVersion={}",
            task_path(&task),
            cleared["version"]
        ),
        "user-a",
        None,
        204,
    )
    .await;
    api(&app, "GET", &task_path(&task), "user-a", None, 404).await;
    api(
        &app,
        "DELETE",
        &format!("{}?expectedVersion=3", task_path(&task)),
        "user-a",
        None,
        404,
    )
    .await;
    fixture.cleanup().await;
}

#[tokio::test]
async fn concurrent_patches_have_one_winner() {
    let Some((app, fixture)) = setup("tasks_concurrent_patch").await else {
        return;
    };
    let task = create(&app, "user-a", json!({"title":"Original"})).await;
    let path = task_path(&task);
    let (a, b) = tokio::join!(
        request(
            &app,
            "PATCH",
            &path,
            Some("user-a"),
            Some(json!({"expectedVersion":1,"title":"A"}))
        ),
        request(
            &app,
            "PATCH",
            &path,
            Some("user-a"),
            Some(json!({"expectedVersion":1,"title":"B"}))
        )
    );
    let mut statuses = [a.status().as_u16(), b.status().as_u16()];
    statuses.sort();
    assert_eq!(statuses, [200, 409]);
    assert_eq!(get(&app, "user-a", &task).await["version"], 2);
    fixture.cleanup().await;
}
