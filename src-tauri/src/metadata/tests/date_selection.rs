use crate::{
    metadata::{select_metadata_date, RawMetadataDates},
    models::{DateKind, FileKind},
};

fn empty_raw_dates() -> RawMetadataDates {
    RawMetadataDates {
        source_file: None,
        sub_sec_date_time_original: None,
        date_time_original: None,
        create_date: None,
        modify_date: None,
        media_create_date: None,
        track_create_date: None,
        file_modify_date: None,
        device_name: None,
        gps: None,
    }
}

#[test]
fn chooses_photo_priority_order() {
    let mut raw = empty_raw_dates();
    raw.date_time_original = Some("2026:03:05 07:20:17".to_string());
    raw.create_date = Some("2024:01:01 01:02:03".to_string());

    let selected = select_metadata_date(&FileKind::Photo, &raw)
        .expect("selection should succeed")
        .expect("a date should be selected");

    assert_eq!(selected.chosen_date_source, "DateTimeOriginal");
    assert!(matches!(selected.date_kind, DateKind::TimezoneUnknown));
    assert_eq!(selected.raw_metadata_date, "2026:03:05 07:20:17");
}

#[test]
fn chooses_video_priority_order() {
    let mut raw = empty_raw_dates();
    raw.track_create_date = Some("2026:03:05 07:20:17".to_string());
    raw.create_date = Some("2025:01:01 00:00:00".to_string());

    let selected = select_metadata_date(&FileKind::Video, &raw)
        .expect("selection should succeed")
        .expect("a date should be selected");

    assert_eq!(selected.chosen_date_source, "TrackCreateDate");
    assert!(matches!(selected.date_kind, DateKind::TimezoneUnknown));
}

#[test]
fn marks_file_modify_date_as_fallback() {
    let mut raw = empty_raw_dates();
    raw.file_modify_date = Some("2026:03:05 07:20:17".to_string());

    let selected = select_metadata_date(&FileKind::Photo, &raw)
        .expect("selection should succeed")
        .expect("a date should be selected");

    assert_eq!(selected.chosen_date_source, "FileModifyDate");
    assert!(matches!(selected.date_kind, DateKind::Fallback));
}

#[test]
fn parses_timezone_aware_dates() {
    let mut raw = empty_raw_dates();
    raw.sub_sec_date_time_original = Some("2026:03:05 07:20:17.616+11:00".to_string());

    let selected = select_metadata_date(&FileKind::Photo, &raw)
        .expect("selection should succeed")
        .expect("a date should be selected");

    assert_eq!(selected.chosen_date_source, "SubSecDateTimeOriginal");
    assert!(matches!(selected.date_kind, DateKind::TimezoneAware));
    // The date is shown in the computer's time zone, so the expected text depends on it.
    let local = chrono::DateTime::parse_from_rfc3339("2026-03-05T07:20:17.616+11:00")
        .expect("valid date")
        .with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    assert!(selected.chosen_date.starts_with(&local));
}

#[test]
fn parses_timezone_less_dates_as_local_wall_time() {
    let mut raw = empty_raw_dates();
    raw.create_date = Some("2026:03:05 07:20:17".to_string());

    let selected = select_metadata_date(&FileKind::Photo, &raw)
        .expect("selection should succeed")
        .expect("a date should be selected");

    assert_eq!(selected.chosen_date, "2026-03-05 07:20:17");
    assert!(matches!(selected.date_kind, DateKind::TimezoneUnknown));
}
