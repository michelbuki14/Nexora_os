//! Time utilities for consistent time handling across services.

use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

/// Current UTC timestamp as RFC3339 string.
pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

/// Current UTC timestamp as Unix seconds.
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Current UTC timestamp as Unix milliseconds.
pub fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Parse RFC3339 string to DateTime<Utc>.
pub fn parse_rfc3339(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc))
}

/// Convert Unix seconds to DateTime<Utc>.
pub fn from_unix(secs: u64) -> DateTime<Utc> {
    Utc.timestamp_opt(secs as i64, 0)
        .single()
        .unwrap_or_else(Utc::now)
}

/// Convert Unix milliseconds to DateTime<Utc>.
pub fn from_unix_ms(ms: u64) -> DateTime<Utc> {
    let secs = ms / 1000;
    let nsecs = ((ms % 1000) * 1_000_000) as u32;
    Utc.timestamp_opt(secs as i64, nsecs)
        .single()
        .unwrap_or_else(Utc::now)
}

/// Add duration to current time.
pub fn add_duration(duration: Duration) -> DateTime<Utc> {
    Utc::now() + duration
}

/// Subtract duration from current time.
pub fn sub_duration(duration: Duration) -> DateTime<Utc> {
    Utc::now() - duration
}

/// Format duration as human-readable string.
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.num_seconds();
    if secs >= 86400 {
        format!("{}d", secs / 86400)
    } else if secs >= 3600 {
        format!("{}h", secs / 3600)
    } else if secs >= 60 {
        format!("{}m", secs / 60)
    } else {
        format!("{}s", secs)
    }
}

/// Calculate age from birth date.
pub fn calculate_age(birth_date: DateTime<Utc>) -> u32 {
    let now = Utc::now();
    let years = now.year() - birth_date.year();
    if now.month() < birth_date.month()
        || (now.month() == birth_date.month() && now.day() < birth_date.day())
    {
        years.saturating_sub(1) as u32
    } else {
        years as u32
    }
}

/// Check if a date is in the past.
pub fn is_past(date: DateTime<Utc>) -> bool {
    date < Utc::now()
}

/// Check if a date is in the future.
pub fn is_future(date: DateTime<Utc>) -> bool {
    date > Utc::now()
}

/// Get start of day in UTC.
pub fn start_of_day(date: DateTime<Utc>) -> DateTime<Utc> {
    date.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc()
}

/// Get end of day in UTC.
pub fn end_of_day(date: DateTime<Utc>) -> DateTime<Utc> {
    date.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc()
}

/// Get start of month in UTC.
pub fn start_of_month(date: DateTime<Utc>) -> DateTime<Utc> {
    date.with_day(1)
        .unwrap()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
}

/// Get end of month in UTC.
pub fn end_of_month(date: DateTime<Utc>) -> DateTime<Utc> {
    let next_month = if date.month() == 12 {
        date.with_year(date.year() + 1)
            .unwrap()
            .with_month(1)
            .unwrap()
    } else {
        date.with_month(date.month() + 1).unwrap()
    };
    next_month
        .with_day(1)
        .unwrap()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        - Duration::seconds(1)
}
