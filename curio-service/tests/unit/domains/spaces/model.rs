use super::model::*;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use serde_json::json;

fn input(name: &str, description: Option<&str>) -> CreateSpaceInput {
    CreateSpaceInput {
        name: name.to_owned(),
        description: description.map(str::to_owned),
    }
}

#[test]
fn trims_values_preserves_case_and_generates_ids() {
    let value = input("  Research  ", Some("  Notes  "));
    let parsed = NewSpace::parse(&value).unwrap();
    assert_eq!(parsed.name(), "Research");
    assert_eq!(parsed.description(), Some("Notes"));
    assert_eq!(
        uuid::Uuid::parse_str(parsed.id())
            .unwrap()
            .get_version_num(),
        4
    );
    assert_ne!(parsed.id(), NewSpace::parse(&value).unwrap().id());
    let empty = input(" Research ", Some("   "));
    assert_eq!(NewSpace::parse(&empty).unwrap().description(), Some(""));
    assert_eq!(
        NewSpace::parse(&input("Research", None))
            .unwrap()
            .description(),
        None
    );
}

#[test]
fn validates_character_counts_and_controls_before_trimming() {
    for name in [
        "".to_owned(),
        "  ".into(),
        "a".repeat(121),
        "é".repeat(121),
        "\nName".into(),
        "Name\t".into(),
        "Na\u{7f}me".into(),
    ] {
        assert!(matches!(
            NewSpace::parse(&input(&name, None)),
            Err(SpaceError::Invalid(_))
        ));
    }
    for description in [
        "a".repeat(2001),
        "é".repeat(2001),
        "\nText".into(),
        "Text\r".into(),
        "x\0y".into(),
    ] {
        assert!(matches!(
            NewSpace::parse(&input("Name", Some(&description))),
            Err(SpaceError::Invalid(_))
        ));
    }
    assert!(NewSpace::parse(&input(&"é".repeat(120), Some(&"é".repeat(2000)))).is_ok());
}

#[test]
fn create_disallows_client_identity_and_unknown_fields() {
    for field in ["ownerId", "userId", "id", "createdAt", "extra"] {
        let mut value = json!({"name":"Research"});
        value[field] = json!("injected");
        assert!(serde_json::from_value::<CreateSpaceInput>(value).is_err());
    }
}

#[test]
fn page_limits_are_bounded() {
    assert_eq!(
        SpacePage::parse(ListSpacesQuery::default())
            .unwrap()
            .limit(),
        50
    );
    for limit in [0, 101, u32::MAX] {
        assert!(matches!(
            SpacePage::parse(ListSpacesQuery {
                limit: Some(limit),
                cursor: None
            }),
            Err(SpaceError::InvalidQuery)
        ));
    }
    for limit in [1, 100] {
        assert_eq!(
            SpacePage::parse(ListSpacesQuery {
                limit: Some(limit),
                cursor: None
            })
            .unwrap()
            .limit(),
            limit as usize
        );
    }
}

#[test]
fn cursor_round_trip_and_untrusted_payload_validation() {
    let timestamp = DateTime::parse_from_rfc3339("2026-09-25T12:30:00.123Z")
        .unwrap()
        .with_timezone(&Utc);
    let space = Space {
        id: "abcdef01-2345-4678-89ab-cdef01234567".into(),
        name: "Research".into(),
        description: None,
        created_at: timestamp,
        updated_at: timestamp,
    };
    let encoded = SpaceCursor::encode(&space).unwrap();
    let page = SpacePage::parse(ListSpacesQuery {
        limit: None,
        cursor: Some(encoded),
    })
    .unwrap();
    assert_eq!(page.cursor().unwrap().created_at(), timestamp);
    assert_eq!(page.cursor().unwrap().id(), space.id);

    let mut invalid = vec![
        "".into(),
        "not-base64!".into(),
        "a".repeat(257),
        URL_SAFE_NO_PAD.encode("not json"),
    ];
    for value in [
        json!({}),
        json!({"created_at":i64::MAX,"id":space.id}),
        json!({"created_at":0,"id":"not-an-id"}),
        json!({"created_at":0,"id":space.id.to_uppercase()}),
        json!({"created_at":"0","id":space.id}),
        json!({"created_at":0,"id":space.id,"ownerId":"another-user"}),
    ] {
        invalid.push(URL_SAFE_NO_PAD.encode(value.to_string()));
    }
    for cursor in invalid {
        assert!(matches!(
            SpacePage::parse(ListSpacesQuery {
                limit: None,
                cursor: Some(cursor)
            }),
            Err(SpaceError::InvalidQuery)
        ));
    }
}
