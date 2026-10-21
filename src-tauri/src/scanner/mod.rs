//! Recursive TODO-folder scan that streams discovered and skipped files as events.

mod classification;
mod scanned_file;

#[cfg(test)]
mod tests;

use std::path::PathBuf;

use tokio_util::sync::CancellationToken;
use walkdir::WalkDir;

use crate::{
    events,
    models::{ScanErrorEvent, ScanFileEvent, ScanFinishedEvent, ScanStartedEvent, ScanStatus},
};
use scanned_file::record_for_path;

pub(crate) fn scan_folder(
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

    let mut totals = ScanFinishedEvent {
        discovered: 0,
        skipped: 0,
        errors: 0,
        cancelled: false,
    };

    for entry in WalkDir::new(&root) {
        if cancellation_token.is_cancelled() {
            totals.cancelled = true;
            break;
        }

        let entry = match entry {
            Ok(value) => value,
            Err(error) => {
                totals.errors += 1;
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

        let record = record_for_path(&root, &entry.into_path());
        if matches!(record.scan_status, ScanStatus::Discovered) {
            totals.discovered += 1;
            let _ = events::emit_scan_file(&app, ScanFileEvent { record });
        } else {
            totals.skipped += 1;
            let _ = events::emit_scan_skipped(&app, ScanFileEvent { record });
        }
    }

    events::emit_scan_finished(&app, totals)
}
