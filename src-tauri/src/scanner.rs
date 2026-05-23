use std::path::{Path, PathBuf};

use tokio_util::sync::CancellationToken;
use walkdir::WalkDir;

use crate::{
    events,
    models::{
        FileKind, FileRecord, ScanErrorEvent, ScanFileEvent, ScanFinishedEvent, ScanStartedEvent,
        ScanStatus,
    },
};

const PHOTO_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "heic", "heif", "tif", "tiff", "webp",
];
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mov", "m4v", "avi", "mts", "m2ts"];
const EXPLICIT_UNSUPPORTED_EXTENSIONS: &[&str] = &["txt", "json", "aae", "xmp"];

pub fn classify_extension(extension: &str) -> FileKind {
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

pub fn is_in_processed(root: &Path, candidate: &Path) -> bool {
    let Ok(relative) = candidate.strip_prefix(root) else {
        return false;
    };

    relative
        .components()
        .any(|component| {
            component
                .as_os_str()
                .to_string_lossy()
                .eq_ignore_ascii_case("Processed")
        })
}

pub fn scan_folder(
    app: tauri::AppHandle,
    root: PathBuf,
    cancellation_token: CancellationToken,
) -> Result<(), String> {
    events::emit_scan_started(
        &app,
        ScanStartedEvent {
            root: root.to_string_lossy().to_string(),
        },
    )?;

    let mut discovered = 0_u64;
    let mut skipped = 0_u64;
    let mut errors = 0_u64;
    let mut cancelled = false;

    for entry in WalkDir::new(&root).into_iter() {
        if cancellation_token.is_cancelled() {
            cancelled = true;
            break;
        }

        let entry = match entry {
            Ok(value) => value,
            Err(error) => {
                errors += 1;
                let path = error
                    .path()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "<unknown>".to_string());
                let _ = events::emit_scan_error(
                    &app,
                    ScanErrorEvent {
                        path,
                        error: error.to_string(),
                    },
                );
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let absolute_path = entry.into_path();
        if is_in_processed(&root, &absolute_path) {
            skipped += 1;
            let record = build_file_record(
                &root,
                &absolute_path,
                FileKind::Unsupported,
                ScanStatus::SkippedProcessed,
            );
            let _ = events::emit_scan_skipped(&app, ScanFileEvent { record });
            continue;
        }

        let extension = absolute_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default();
        let kind = classify_extension(extension);

        if matches!(kind, FileKind::Photo | FileKind::Video) {
            discovered += 1;
            let record = build_file_record(&root, &absolute_path, kind, ScanStatus::Discovered);
            let _ = events::emit_scan_file(&app, ScanFileEvent { record });
        } else {
            skipped += 1;
            let record = build_file_record(
                &root,
                &absolute_path,
                kind,
                ScanStatus::SkippedUnsupported,
            );
            let _ = events::emit_scan_skipped(&app, ScanFileEvent { record });
        }
    }

    events::emit_scan_finished(
        &app,
        ScanFinishedEvent {
            discovered,
            skipped,
            errors,
            cancelled,
        },
    )
}

fn build_file_record(root: &Path, absolute_path: &Path, kind: FileKind, status: ScanStatus) -> FileRecord {
    let relative_path = absolute_path
        .strip_prefix(root)
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|_| absolute_path.to_string_lossy().to_string());

    let id = absolute_path.to_string_lossy().to_string();
    let extension = absolute_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();

    FileRecord {
        id,
        source_path: absolute_path.to_string_lossy().to_string(),
        relative_path,
        extension,
        kind,
        scan_status: status,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{classify_extension, is_in_processed};
    use crate::models::FileKind;

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
        let root = Path::new(r"C:\todo");
        assert!(is_in_processed(
            root,
            Path::new(r"C:\todo\Processed\2026\01\example.jpg")
        ));
        assert!(is_in_processed(
            root,
            Path::new(r"C:\todo\nested\processed\image.jpg")
        ));
        assert!(!is_in_processed(root, Path::new(r"C:\todo\nested\inbox\image.jpg")));
    }
}
