//! Wire-format parsing and normalization for calendar events.
//!
//! Two representations of the same time live side by side. The wire strings
//! (`startDate` / `endDate`) are stored verbatim because the frontend
//! classifies events by their *format*: a bare `YYYY-MM-DD` is all-day, a full
//! timestamp is timed, and a timestamp with no end is a milestone. Normalizing
//! those strings would silently turn every all-day event into a midnight block.
//!
//! Those same strings cannot drive range queries. Compared lexicographically,
//! `"2026-06-01" < "2026-06-01T00:00:00.000Z"`, so a naive overlap test would
//! drop an all-day event sitting on the first day of a range. `starts_at` and
//! `ends_at` are derived here, once, purely so the query is correct.

use chrono::{DateTime, Days, Duration, NaiveDate, TimeZone, Utc};

/// A milestone — a timestamp with no end — gets this nominal duration so that
/// `ends_at > starts_at` holds for every stored row, which is what lets the
/// overlap predicate be a single expression with no special cases.
const NOMINAL_MILESTONE_MINUTES: i64 = 1;

/// Which of the two wire formats a date string is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireFormat {
    /// A bare `YYYY-MM-DD`, meaning an all-day event on a floating calendar day.
    Date,
    /// A full RFC 3339 timestamp with an explicit offset.
    Timestamp,
}

/// The normalized half-open UTC interval `[starts_at, ends_at)` for an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedInterval {
    pub starts_at: String,
    pub ends_at: String,
}

/// Renders an instant in the one canonical, fixed-width UTC form used for every
/// stored bound. Fixed width matters: it is what makes the stored strings sort
/// lexicographically in chronological order, so SQLite's `<` and `>` on TEXT
/// columns are real time comparisons.
pub fn format_instant(instant: DateTime<Utc>) -> String {
    instant.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Classifies a wire date string, returning `None` when it is neither format.
pub fn wire_format(value: &str) -> Option<WireFormat> {
    if parse_date(value).is_some() {
        return Some(WireFormat::Date);
    }
    if parse_timestamp(value).is_some() {
        return Some(WireFormat::Timestamp);
    }
    None
}

/// Parses a range bound. Bounds arrive from the client already converted to UTC
/// from its own local view; a bare date is accepted as that day's UTC midnight.
pub fn parse_bound(value: &str) -> Option<DateTime<Utc>> {
    if let Some(date) = parse_date(value) {
        return Some(start_of_day(date));
    }
    parse_timestamp(value)
}

/// Derives the half-open UTC interval an event occupies.
///
/// `all_day` must agree with the format `start_date` was written in rather than
/// the server quietly picking one, and `end_date`, when present, must be in the
/// same format as `start_date` and must not fall before it.
///
/// Returns a caller-facing message on failure; every one of them is a `422`.
pub fn normalize_event(
    all_day: bool,
    start_date: &str,
    end_date: Option<&str>,
) -> Result<NormalizedInterval, &'static str> {
    let format = wire_format(start_date).ok_or(
        "A start date must be either YYYY-MM-DD or a timestamp with an explicit UTC offset.",
    )?;

    if all_day != (format == WireFormat::Date) {
        return Err(
            "An all-day event needs a YYYY-MM-DD date, and a timed event needs a full timestamp.",
        );
    }

    if let Some(end_date) = end_date
        && wire_format(end_date) != Some(format)
    {
        return Err("The end date must be written in the same format as the start date.");
    }

    match format {
        WireFormat::Date => normalize_all_day(start_date, end_date),
        WireFormat::Timestamp => normalize_timed(start_date, end_date),
    }
}

/// All-day `end_date` is *inclusive* on the wire — "through the 25th" — and
/// exclusive in `ends_at`, so it advances by one day. A missing end means the
/// single day the event starts on.
fn normalize_all_day(
    start_date: &str,
    end_date: Option<&str>,
) -> Result<NormalizedInterval, &'static str> {
    let start = parse_date(start_date).ok_or("The start date could not be parsed.")?;
    let last_day = match end_date {
        Some(end_date) => parse_date(end_date).ok_or("The end date could not be parsed.")?,
        None => start,
    };

    if last_day < start {
        return Err("The end date cannot fall before the start date.");
    }

    let exclusive_end = last_day
        .checked_add_days(Days::new(1))
        .ok_or("That end date is out of range.")?;

    Ok(NormalizedInterval {
        starts_at: format_instant(start_of_day(start)),
        ends_at: format_instant(start_of_day(exclusive_end)),
    })
}

/// A timed event runs to its end timestamp. A milestone — no end at all — and a
/// zero-length event both get the nominal duration, so the `ends_at > starts_at`
/// invariant holds either way.
fn normalize_timed(
    start_date: &str,
    end_date: Option<&str>,
) -> Result<NormalizedInterval, &'static str> {
    let starts_at = parse_timestamp(start_date).ok_or("The start date could not be parsed.")?;
    let nominal_end = starts_at + Duration::minutes(NOMINAL_MILESTONE_MINUTES);

    let ends_at = match end_date {
        Some(end_date) => {
            let ends_at = parse_timestamp(end_date).ok_or("The end date could not be parsed.")?;
            if ends_at < starts_at {
                return Err("The end date cannot fall before the start date.");
            }
            ends_at.max(nominal_end)
        }
        None => nominal_end,
    };

    Ok(NormalizedInterval {
        starts_at: format_instant(starts_at),
        ends_at: format_instant(ends_at),
    })
}

/// All-day dates are floating — `"2026-06-22"` means that calendar day wherever
/// the reader is — and UTC midnight is where this anchors them.
fn start_of_day(date: NaiveDate) -> DateTime<Utc> {
    Utc.from_utc_datetime(&date.and_time(chrono::NaiveTime::MIN))
}

/// Strictly `YYYY-MM-DD`. The explicit shape check rejects the loose forms
/// `NaiveDate` would otherwise accept, such as `2026-6-1`.
fn parse_date(value: &str) -> Option<NaiveDate> {
    let bytes = value.as_bytes();
    let shaped = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && [0, 1, 2, 3, 5, 6, 8, 9]
            .iter()
            .all(|index| bytes[*index].is_ascii_digit());

    shaped
        .then(|| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
        .flatten()
}

/// An explicit offset is required. A timestamp without one is ambiguous, and
/// guessing a zone here is exactly the kind of silent normalization this module
/// exists to prevent.
fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|instant| instant.with_timezone(&Utc))
}
