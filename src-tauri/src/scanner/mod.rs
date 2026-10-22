//! Recursive TODO-folder scan that streams discovered and skipped files as events.

mod active_scan;
mod classification;
mod file_signature;
mod media_identity;
mod scanned_file;

#[cfg(test)]
mod tests;

use std::path::Path;

use tokio_util::sync::CancellationToken;
use walkdir::WalkDir;

use crate::{
    events::{self, EventSink},
    models::{ScanErrorEvent, ScanFileEvent, ScanFinishedEvent, ScanStartedEvent, ScanStatus},
};
pub(crate) use active_scan::ActiveScan;
pub(crate) use scanned_file::record_for_path;

pub(crate) fn scan_folder(
    sink: &impl EventSink,
    root: &Path,
    cancellation_token: &CancellationToken,
) {
    events::scan_started(
        sink,
        ScanStartedEvent {
            root: root.to_string_lossy().to_string(),
        },
    );

    let mut totals = ScanFinishedEvent {
        discovered: 0,
        skipped: 0,
        errors: 0,
        cancelled: false,
    };

    for entry in WalkDir::new(root) {
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
                events::scan_error(
                    sink,
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

        let record = record_for_path(root, &entry.into_path());
        if matches!(record.scan_status, ScanStatus::Discovered) {
            totals.discovered += 1;
            events::scan_file(sink, ScanFileEvent { record });
        } else {
            totals.skipped += 1;
            events::scan_skipped(sink, ScanFileEvent { record });
        }
    }

    events::scan_finished(sink, totals);
}
