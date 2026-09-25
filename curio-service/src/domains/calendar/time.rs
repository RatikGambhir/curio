use chrono::{DateTime, Days, Duration, NaiveDate, TimeZone, Utc};

const NOMINAL_MILESTONE_MINUTES: i64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WireFormat {
    Date,
    Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedInterval {
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

#[cfg(test)]
pub fn format_instant(instant: DateTime<Utc>) -> String {
    instant.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

pub fn parse_bound(value: &str) -> Option<DateTime<Utc>> {
    if let Some(date) = parse_date(value) {
        return Some(start_of_day(date));
    }
    parse_timestamp(value)
}

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

pub(super) fn wire_format(value: &str) -> Option<WireFormat> {
    if parse_date(value).is_some() {
        return Some(WireFormat::Date);
    }
    if parse_timestamp(value).is_some() {
        return Some(WireFormat::Timestamp);
    }
    None
}

fn normalize_all_day(
    start_date: &str,
    end_date: Option<&str>,
) -> Result<NormalizedInterval, &'static str> {
    let start = parse_date(start_date).ok_or("The start date could not be parsed.")?;
    let exclusive_end = match end_date {
        Some(end_date) => {
            let end = parse_date(end_date).ok_or("The end date could not be parsed.")?;
            if end <= start {
                return Err("The end date must fall after the start date.");
            }
            end
        }
        None => start
            .checked_add_days(Days::new(1))
            .ok_or("That start date is out of range.")?,
    };

    Ok(NormalizedInterval {
        starts_at: start_of_day(start),
        ends_at: start_of_day(exclusive_end),
    })
}

fn normalize_timed(
    start_date: &str,
    end_date: Option<&str>,
) -> Result<NormalizedInterval, &'static str> {
    let starts_at = parse_timestamp(start_date).ok_or("The start date could not be parsed.")?;

    let ends_at = match end_date {
        Some(end_date) => {
            let ends_at = parse_timestamp(end_date).ok_or("The end date could not be parsed.")?;
            if ends_at <= starts_at {
                return Err("The end date must fall after the start date.");
            }
            ends_at
        }
        None => starts_at
            .checked_add_signed(Duration::minutes(NOMINAL_MILESTONE_MINUTES))
            .ok_or("That start date is out of range.")?,
    };

    Ok(NormalizedInterval { starts_at, ends_at })
}

fn start_of_day(date: NaiveDate) -> DateTime<Utc> {
    Utc.from_utc_datetime(&date.and_time(chrono::NaiveTime::MIN))
}

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

fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|instant| instant.with_timezone(&Utc))
}
