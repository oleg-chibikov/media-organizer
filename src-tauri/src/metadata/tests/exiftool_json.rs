use crate::metadata::exiftool_json::parse_exiftool_json;

#[test]
fn parses_known_exiftool_fields() {
    let json = r#"[{
        "SourceFile": "C:/todo/example.jpg",
        "SubSecDateTimeOriginal": "2026:03:05 07:20:17.616+11:00",
        "DateTimeOriginal": "2026:03:05 07:20:17",
        "CreateDate": "2026:03:05 07:20:17"
    }]"#;

    let parsed = parse_exiftool_json(json).expect("json should parse");
    assert_eq!(
        parsed.sub_sec_date_time_original.as_deref(),
        Some("2026:03:05 07:20:17.616+11:00")
    );
    assert_eq!(
        parsed.date_time_original.as_deref(),
        Some("2026:03:05 07:20:17")
    );
    assert_eq!(parsed.device_name.as_deref(), None);
    assert_eq!(parsed.media_create_date, None);
}

#[test]
fn parses_device_name_from_video_fallback_tags() {
    let json = r#"[{
        "SourceFile": "C:/todo/example.mp4",
        "MediaCreateDate": "2026:03:05 07:20:17",
        "DeviceModelName": "Pixel 8 Pro"
    }]"#;

    let parsed = parse_exiftool_json(json).expect("json should parse");
    assert_eq!(parsed.device_name.as_deref(), Some("Pixel 8 Pro"));
    assert_eq!(
        parsed.media_create_date.as_deref(),
        Some("2026:03:05 07:20:17")
    );
}

#[test]
fn rejects_empty_json_payload() {
    let result = parse_exiftool_json("[]");
    assert!(result.is_err());
}
