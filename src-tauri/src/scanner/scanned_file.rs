use std::path::Path;

use super::classification::{classify_extension, is_in_processed};
use crate::{
    filename,
    models::{FileKind, FileRecord, ScanStatus},
};

/// Classifies one scanned file as discovered media or a skipped item.
pub(super) fn record_for_path(root: &Path, absolute_path: &Path) -> FileRecord {
    if is_in_processed(root, absolute_path) {
        return build_file_record(
            root,
            absolute_path,
            FileKind::Unsupported,
            ScanStatus::SkippedProcessed,
        );
    }

    let kind = classify_extension(file_extension(absolute_path));
    let status = if matches!(kind, FileKind::Photo | FileKind::Video) {
        ScanStatus::Discovered
    } else {
        ScanStatus::SkippedUnsupported
    };
    build_file_record(root, absolute_path, kind, status)
}

fn build_file_record(
    root: &Path,
    absolute_path: &Path,
    kind: FileKind,
    status: ScanStatus,
) -> FileRecord {
    let relative_path = absolute_path
        .strip_prefix(root)
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|_| absolute_path.to_string_lossy().to_string());
    let file_name = absolute_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    FileRecord {
        id: absolute_path.to_string_lossy().to_string(),
        source_path: absolute_path.to_string_lossy().to_string(),
        relative_path,
        extension: file_extension(absolute_path).to_string(),
        original_name: filename::parse_original_name(file_name),
        kind,
        scan_status: status,
    }
}

fn file_extension(path: &Path) -> &str {
    path.extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
}
