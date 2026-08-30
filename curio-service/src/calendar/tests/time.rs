//! The normalization rules, tested without a database or a router in sight.
//!
//! This is where the wire-format contract actually lives: what counts as
//! all-day, how an inclusive end becomes an exclusive instant, and why every
//! stored row ends after it starts.

use crate::calendar::time::*;

#[test]
fn a_date_only_string_normalizes_to_utc_midnight() {
    let interval = normalize_event(true, "2026-06-22", None).unwrap();

    assert_eq!(interval.starts_at, "2026-06-22T00:00:00.000Z");
    assert_eq!(interval.ends_at, "2026-06-23T00:00:00.000Z");
}

#[test]
fn an_inclusive_all_day_end_becomes_an_exclusive_instant_one_day_later() {
    let interval = normalize_event(true, "2026-06-22", Some("2026-06-25")).unwrap();

    assert_eq!(interval.starts_at, "2026-06-22T00:00:00.000Z");
    assert_eq!(interval.ends_at, "2026-06-26T00:00:00.000Z");
}

#[test]
fn a_milestone_ends_after_it_starts() {
    let interval = normalize_event(false, "2026-06-22T14:00:00.000Z", None).unwrap();

    assert_eq!(interval.starts_at, "2026-06-22T14:00:00.000Z");
    assert_eq!(interval.ends_at, "2026-06-22T14:01:00.000Z");
    assert!(interval.ends_at > interval.starts_at);
}

#[test]
fn a_timestamp_round_trips_unchanged() {
    let interval = normalize_event(
        false,
        "2026-06-22T14:00:00.000Z",
        Some("2026-06-22T15:30:00.000Z"),
    )
    .unwrap();

    assert_eq!(interval.starts_at, "2026-06-22T14:00:00.000Z");
    assert_eq!(interval.ends_at, "2026-06-22T15:30:00.000Z");
}

#[test]
fn a_non_utc_offset_is_converted_to_utc() {
    let interval = normalize_event(false, "2026-06-22T07:00:00-07:00", None).unwrap();

    assert_eq!(interval.starts_at, "2026-06-22T14:00:00.000Z");
}

#[test]
fn a_zero_length_timed_event_still_ends_after_it_starts() {
    let interval = normalize_event(
        false,
        "2026-06-22T14:00:00.000Z",
        Some("2026-06-22T14:00:00.000Z"),
    )
    .unwrap();

    assert_eq!(interval.ends_at, "2026-06-22T14:01:00.000Z");
}

#[test]
fn every_normalized_interval_sorts_chronologically_as_a_string() {
    let earlier = normalize_event(true, "2026-06-01", None).unwrap();
    let later = normalize_event(false, "2026-06-01T00:00:00.000Z", None).unwrap();

    assert!(earlier.starts_at <= later.starts_at);
    assert_eq!(earlier.starts_at, later.starts_at);
}

#[test]
fn all_day_must_agree_with_the_format_it_was_given() {
    assert!(normalize_event(true, "2026-06-22T14:00:00.000Z", None).is_err());
    assert!(normalize_event(false, "2026-06-22", None).is_err());
}

#[test]
fn a_mixed_format_end_is_rejected() {
    assert!(normalize_event(true, "2026-06-22", Some("2026-06-25T00:00:00.000Z")).is_err());
    assert!(normalize_event(false, "2026-06-22T14:00:00.000Z", Some("2026-06-25")).is_err());
}

#[test]
fn an_end_before_the_start_is_rejected() {
    assert!(normalize_event(true, "2026-06-22", Some("2026-06-21")).is_err());
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
fn unparseable_and_loosely_written_dates_are_rejected() {
    assert_eq!(wire_format("2026-6-1"), None);
    assert_eq!(wire_format("22/06/2026"), None);
    assert_eq!(wire_format("2026-02-30"), None);
    assert_eq!(wire_format("tomorrow"), None);
    assert_eq!(wire_format(""), None);
}

#[test]
fn a_timestamp_without_an_offset_is_not_a_valid_wire_value() {
    assert_eq!(wire_format("2026-06-22T14:00:00"), None);
}

#[test]
fn bounds_accept_both_wire_formats() {
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
