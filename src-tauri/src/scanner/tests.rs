use std::path::Path;

use crate::models::FileKind;
use crate::scanner::classification::{classify_extension, is_in_processed};

#[test]
fn classifies_supported_extensions() {
    assert!(matches!(classify_extension("jpg"), FileKind::Photo));
    assert!(matches!(classify_extension("HEIC"), FileKind::Photo));
    assert!(matches!(classify_extension("m2ts"), FileKind::Video));
}

#[test]
fn classifies_unsupported_and_unknown_extensions() {
    assert!(matches!(classify_extension("xmp"), FileKind::Unsupported));
    assert!(matches!(classify_extension(""), FileKind::Unknown));
    assert!(matches!(classify_extension("raw"), FileKind::Unknown));
}

#[test]
fn detects_processed_segment_in_relative_path() {
    let root = Path::new("todo");
    assert!(is_in_processed(
        root,
        &root
            .join("Processed")
            .join("2026")
            .join("01")
            .join("example.jpg")
    ));
    assert!(is_in_processed(
        root,
        &root.join("nested").join("processed").join("image.jpg")
    ));
    assert!(!is_in_processed(
        root,
        &root.join("nested").join("inbox").join("image.jpg")
    ));
}

#[test]
fn finishing_an_older_scan_keeps_the_newer_scan_cancellable() {
    let active_scan = crate::scanner::ActiveScan::default();
    let older = active_scan.begin();
    let newer = active_scan.begin();

    active_scan.finish(&older);
    active_scan.cancel();

    assert!(newer.token.is_cancelled());
    assert!(!older.token.is_cancelled());
}

#[test]
fn streams_discovered_and_skipped_files_then_totals() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("Processed")).expect("create Processed");
    std::fs::write(root.join("photo.jpg"), b"photo").expect("write photo");
    std::fs::write(root.join("notes.txt"), b"notes").expect("write notes");
    std::fs::write(root.join("Processed").join("done.jpg"), b"done").expect("write done");
    let sink = crate::test_support::events::RecordingSink::default();

    crate::scanner::scan_folder(&sink, root, &tokio_util::sync::CancellationToken::new());

    let names = sink.names();
    assert_eq!(names.first(), Some(&crate::events::SCAN_STARTED));
    assert_eq!(names.last(), Some(&crate::events::SCAN_FINISHED));
    let finished = &sink.payloads(crate::events::SCAN_FINISHED)[0];
    assert_eq!(finished["discovered"], 1);
    assert_eq!(finished["skipped"], 2);
    assert_eq!(finished["cancelled"], false);
}
