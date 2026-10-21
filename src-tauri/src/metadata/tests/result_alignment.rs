use std::path::PathBuf;

use crate::metadata::{exiftool_json::parse_exiftool_json_batch, result_alignment::align_results};

#[test]
fn matches_results_by_source_file_when_exiftool_skips_a_file() {
    let paths = [r"C:\todo\gone.jpg", r"C:\todo\kept.jpg"].map(PathBuf::from);
    let json = r#"[{"SourceFile": "C:/todo/kept.jpg", "CreateDate": "2026:03:05 07:20:17"}]"#;
    let results = parse_exiftool_json_batch(json).expect("json should parse");

    let aligned = align_results(&paths, results);

    assert!(aligned[0].is_none());
    let kept = aligned[1].as_ref().expect("kept file has a result");
    assert_eq!(kept.create_date.as_deref(), Some("2026:03:05 07:20:17"));
}

#[test]
fn keeps_results_in_request_order_when_every_file_was_read() {
    let paths = ["/todo/a.jpg", "/todo/b.jpg"].map(PathBuf::from);
    let json = r#"[{"SourceFile": "/todo/a.jpg"}, {"SourceFile": "/todo/b.jpg", "Model": "X"}]"#;
    let results = parse_exiftool_json_batch(json).expect("json should parse");

    let aligned = align_results(&paths, results);

    assert_eq!(aligned.len(), 2);
    assert_eq!(
        aligned[1].as_ref().expect("b").device_name.as_deref(),
        Some("X")
    );
}
