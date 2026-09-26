use super::{filter::*, model::*};
use serde_json::json;

#[test]
fn patch_presence_and_validation() {
    let patch: PatchTask =
        serde_json::from_value(json!({"expectedVersion":1,"description":null})).unwrap();
    assert_eq!(patch.description, Field::Null);
    assert_eq!(patch.title, Field::Missing);
    assert!(patch.validate().is_ok());
    assert!(
        serde_json::from_value::<PatchTask>(json!({"expectedVersion":1,"ownerId":"x"})).is_err()
    );
    assert!(
        serde_json::from_value::<PatchTask>(json!({"expectedVersion":1}))
            .unwrap()
            .validate()
            .is_err()
    );
}
#[test]
fn bounds_dates_urls_and_tag_keys() {
    assert!(text(&"é".repeat(240), 240, false).is_ok());
    assert!(text(&"é".repeat(241), 240, false).is_err());
    assert!(text("x\0y", 100, false).is_err());
    assert!(text("line\nnext", 100, true).is_ok());
    assert_eq!(tag_name(" Work ").unwrap(), ("Work".into(), "work".into()));
    assert!(date("2026-02-30").is_err());
    assert!(date("2026-2-01").is_err());
    assert!(
        due(
            Some(date("2026-09-25").unwrap()),
            Some(instant("2026-09-25T00:00:00Z").unwrap()),
            Some("UTC")
        )
        .is_err()
    );
    for url in [
        "javascript:alert(1)",
        "https://user:pass@example.com",
        "https://@example.com",
        "https://example.com/\n",
    ] {
        assert!(
            LinkInput {
                kind: "web".into(),
                label: None,
                url: Some(url.into()),
                note_id: None,
                sort_order: None
            }
            .validate()
            .is_err()
        );
    }
}
#[test]
fn hierarchy_and_completion_rules() {
    let ancestors = vec![TreeNode {
        id: "root".into(),
        depth: 1,
    }];
    assert!(hierarchy("root", &ancestors, 1).is_err());
    assert!(hierarchy("child", &ancestors, 5).is_err());
    assert!(hierarchy("child", &ancestors, 4).is_ok());
    let now = chrono::Utc::now();
    assert_eq!(completion("todo", "done", None, now), Some(now));
    assert_eq!(completion("done", "todo", Some(now), now), None);
    assert_eq!(
        completion(
            "done",
            "done",
            Some(now),
            now + chrono::Duration::seconds(1)
        ),
        Some(now)
    );
}
#[test]
fn filters_reject_incompatible_or_unbounded_input() {
    for value in [
        json!({"statuses":"todo,todo"}),
        json!({"statuses":"todo,"}),
        json!({"limit":101}),
        json!({"sort":"manual"}),
        json!({"sort":"due"}),
        json!({"dueFrom":"2026-09-25"}),
        json!({"dueKind":"date","hasDueDate":false}),
        json!({"dueKind":"date","dueFrom":"2026-09-26","dueBefore":"2026-09-25"}),
    ] {
        assert!(TaskPage::parse(serde_json::from_value(value).unwrap()).is_err());
    }
    assert!(
        TaskPage::parse(
            serde_json::from_value(json!({"sort":"manual","parentId":"none"})).unwrap()
        )
        .is_ok()
    );
}

#[test]
fn cursors_are_bounded_typed_and_filter_specific() {
    let filters = TaskPage::parse(ListQuery::default()).unwrap().filters;
    let valid = TaskCursor {
        filters: filters.clone(),
        key: 0,
        id: "abcdef01-2345-4678-89ab-cdef01234567".into(),
    };
    let cursor = encode(&valid).unwrap();
    assert!(
        TaskPage::parse(serde_json::from_value(json!({"cursor":cursor,"limit":1})).unwrap())
            .is_ok()
    );
    assert!(
        TaskPage::parse(serde_json::from_value(json!({"cursor":cursor,"flagged":true})).unwrap())
            .is_err()
    );
    let mut invalid = vec!["".to_owned(), "not-base64!".into(), "a".repeat(8193)];
    invalid.push(
        encode(&TaskCursor {
            key: i64::MAX,
            ..valid.clone()
        })
        .unwrap(),
    );
    invalid.push(
        encode(&TaskCursor {
            id: "injected".into(),
            ..valid
        })
        .unwrap(),
    );
    for cursor in invalid {
        assert!(
            TaskPage::parse(serde_json::from_value(json!({"cursor":cursor})).unwrap()).is_err()
        );
    }
    let page = CreatedPage::parse(PageQuery::default(), "tags".into()).unwrap();
    let cursor = page
        .next(chrono::Utc::now(), "abcdef01-2345-4678-89ab-cdef01234567")
        .unwrap();
    assert!(
        CreatedPage::parse(
            PageQuery {
                limit: None,
                cursor: Some(cursor)
            },
            "comments:task".into()
        )
        .is_err()
    );
}
