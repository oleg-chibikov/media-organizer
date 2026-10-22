use std::path::Path;

use super::{classification::is_in_processed, media_identity::MediaIdentity};
use crate::models::{FileKind, FileRecord, ScanStatus};

/// Classifies one scanned file as discovered media or a skipped item.
pub(crate) fn record_for_path(root: &Path, absolute_path: &Path) -> FileRecord {
    if is_in_processed(root, absolute_path) {
        let identity = MediaIdentity {
            kind: FileKind::Unsupported,
            ..MediaIdentity::from_name(absolute_path)
        };
        return build_file_record(root, absolute_path, identity, ScanStatus::SkippedProcessed);
    }

    let identity = MediaIdentity::from_name_or_content(absolute_path);
    let status = if matches!(identity.kind, FileKind::Photo | FileKind::Video) {
        ScanStatus::Discovered
    } else {
        ScanStatus::SkippedUnsupported
    };
    build_file_record(root, absolute_path, identity, status)
}

fn build_file_record(
    root: &Path,
    absolute_path: &Path,
    identity: MediaIdentity,
    status: ScanStatus,
) -> FileRecord {
    let relative_path = absolute_path
        .strip_prefix(root)
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|_| absolute_path.to_string_lossy().to_string());

    FileRecord {
        id: absolute_path.to_string_lossy().to_string(),
        source_path: absolute_path.to_string_lossy().to_string(),
        relative_path,
        extension: identity.extension,
        original_name: identity.original_name,
        kind: identity.kind,
        scan_status: status,
    }
}
