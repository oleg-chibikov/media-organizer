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
