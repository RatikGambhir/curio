use crate::calendar::time::*;

fn instant(value: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(value)
        .unwrap()
        .with_timezone(&chrono::Utc)
}

#[test]
fn date_only_values() {
    let interval = normalize_event(true, "2026-06-22", None).unwrap();

    assert_eq!(interval.starts_at, instant("2026-06-22T00:00:00.000Z"));
    assert_eq!(interval.ends_at, instant("2026-06-23T00:00:00.000Z"));
}

#[test]
fn explicit_all_day_ends_are_exclusive() {
    let interval = normalize_event(true, "2026-06-22", Some("2026-06-25")).unwrap();

    assert_eq!(interval.starts_at, instant("2026-06-22T00:00:00.000Z"));
    assert_eq!(interval.ends_at, instant("2026-06-25T00:00:00.000Z"));
}

#[test]
fn milestones_receive_a_queryable_nominal_interval() {
    let milestone = normalize_event(false, "2026-06-22T14:00:00.000Z", None).unwrap();

    assert_eq!(milestone.ends_at, instant("2026-06-22T14:01:00.000Z"));
    assert!(milestone.ends_at > milestone.starts_at);
}

#[test]
fn timestamp_normalization() {
    let utc = normalize_event(
        false,
        "2026-06-22T14:00:00.000Z",
        Some("2026-06-22T15:30:00.000Z"),
    )
    .unwrap();
    let offset = normalize_event(false, "2026-06-22T07:00:00-07:00", None).unwrap();

    assert_eq!(utc.starts_at, instant("2026-06-22T14:00:00.000Z"));
    assert_eq!(utc.ends_at, instant("2026-06-22T15:30:00.000Z"));
    assert_eq!(offset.starts_at, instant("2026-06-22T14:00:00.000Z"));
}

#[test]
fn incompatible_formats() {
    assert!(normalize_event(true, "2026-06-22T14:00:00.000Z", None).is_err());
    assert!(normalize_event(false, "2026-06-22", None).is_err());
    assert!(normalize_event(true, "2026-06-22", Some("2026-06-25T00:00:00Z")).is_err());
    assert!(normalize_event(false, "2026-06-22T14:00:00Z", Some("2026-06-25")).is_err());
}

#[test]
fn invalid_ordering() {
    assert!(normalize_event(true, "2026-06-22", Some("2026-06-22")).is_err());
    assert!(normalize_event(true, "2026-06-22", Some("2026-06-21")).is_err());
    assert!(
        normalize_event(
            false,
            "2026-06-22T14:00:00.000Z",
            Some("2026-06-22T14:00:00.000Z")
        )
        .is_err()
    );
    assert!(
        normalize_event(
            false,
            "2026-06-22T14:00:00.000Z",
            Some("2026-06-22T13:00:00.000Z")
        )
        .is_err()
    );
}

#[test]
fn invalid_wire_values() {
    for value in [
        "2026-6-1",
        "22/06/2026",
        "2026-02-30",
        "tomorrow",
        "",
        "2026-06-22T14:00:00",
    ] {
        assert_eq!(wire_format(value), None);
    }
}

#[test]
fn range_bounds() {
    assert_eq!(
        parse_bound("2026-06-22").map(format_instant).as_deref(),
        Some("2026-06-22T00:00:00.000Z")
    );
    assert_eq!(
        parse_bound("2026-05-31T07:00:00.000Z")
            .map(format_instant)
            .as_deref(),
        Some("2026-05-31T07:00:00.000Z")
    );
    assert!(parse_bound("not-a-date").is_none());
}
