use chrono::{DateTime, Local, LocalResult, NaiveDateTime, TimeZone};

use crate::models::DateKind;

const OUTPUT_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const ZONED_FORMATS: [&str; 4] = [
    "%Y:%m:%d %H:%M:%S%.f%:z",
    "%Y:%m:%d %H:%M:%S%:z",
    "%Y:%m:%d %H:%M:%S%.f%z",
    "%Y:%m:%d %H:%M:%S%z",
];
const UTC_SUFFIX_FORMATS: [&str; 2] = ["%Y:%m:%d %H:%M:%S%.f%:z", "%Y:%m:%d %H:%M:%S%:z"];
const NAIVE_FORMATS: [&str; 2] = ["%Y:%m:%d %H:%M:%S%.f", "%Y:%m:%d %H:%M:%S"];

/// Parses an ExifTool timestamp into local wall time and how trustworthy it is.
pub(super) fn parse_metadata_timestamp(raw: &str, is_fallback: bool) -> Option<(String, DateKind)> {
    if let Some(local) = parse_timezone_aware_to_local(raw) {
        let kind = if is_fallback {
            DateKind::Fallback
        } else {
            DateKind::TimezoneAware
        };
        return Some((local.format(OUTPUT_FORMAT).to_string(), kind));
    }

    if let Some(local_naive) = parse_timezone_unknown_as_local(raw) {
        let kind = if is_fallback {
            DateKind::Fallback
        } else {
            DateKind::TimezoneUnknown
        };
        return Some((local_naive.format(OUTPUT_FORMAT).to_string(), kind));
    }

    None
}

fn parse_timezone_aware_to_local(raw: &str) -> Option<DateTime<Local>> {
    if let Some(parsed) = ZONED_FORMATS
        .iter()
        .find_map(|format| DateTime::parse_from_str(raw, format).ok())
    {
        return Some(parsed.with_timezone(&Local));
    }

    let normalized = format!("{}+00:00", raw.strip_suffix('Z')?);
    UTC_SUFFIX_FORMATS
        .iter()
        .find_map(|format| DateTime::parse_from_str(&normalized, format).ok())
        .map(|parsed| parsed.with_timezone(&Local))
}

fn parse_timezone_unknown_as_local(raw: &str) -> Option<NaiveDateTime> {
    let naive = NAIVE_FORMATS
        .iter()
        .find_map(|format| NaiveDateTime::parse_from_str(raw, format).ok())?;

    match Local.from_local_datetime(&naive) {
        LocalResult::Single(value) => Some(value.naive_local()),
        LocalResult::Ambiguous(first, _) => Some(first.naive_local()),
        LocalResult::None => None,
    }
}
