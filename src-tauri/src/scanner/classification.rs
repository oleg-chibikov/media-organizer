use std::path::Path;

use crate::{models::FileKind, path_safety::path_contains_processed_segment};

const PHOTO_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "heic", "heif", "tif", "tiff", "webp"];
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mov", "m4v", "avi", "mts", "m2ts"];
const EXPLICIT_UNSUPPORTED_EXTENSIONS: &[&str] = &["txt", "json", "aae", "xmp"];

pub(super) fn classify_extension(extension: &str) -> FileKind {
    let normalized = extension.trim_start_matches('.').to_ascii_lowercase();
    if PHOTO_EXTENSIONS.contains(&normalized.as_str()) {
        return FileKind::Photo;
    }
    if VIDEO_EXTENSIONS.contains(&normalized.as_str()) {
        return FileKind::Video;
    }
    if EXPLICIT_UNSUPPORTED_EXTENSIONS.contains(&normalized.as_str()) {
        return FileKind::Unsupported;
    }
    FileKind::Unknown
}

/// Files under any `Processed` folder below the scan root are already organized.
pub(super) fn is_in_processed(root: &Path, candidate: &Path) -> bool {
    candidate
        .strip_prefix(root)
        .is_ok_and(path_contains_processed_segment)
}
