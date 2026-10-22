//! Recursive TODO-folder scan that streams discovered and skipped files as events.

mod active_scan;
mod classification;
mod file_signature;
mod media_identity;
mod record_batch;
mod scanned_file;

#[cfg(test)]
mod tests;

use std::path::Path;

use tokio_util::sync::CancellationToken;
use walkdir::WalkDir;

use crate::{
    events::{self, EventSink},
    models::{FileRecord, ScanErrorEvent, ScanFinishedEvent, ScanStartedEvent, ScanStatus},
};
pub(crate) use active_scan::ActiveScan;
use record_batch::RecordBatch;
pub(crate) use scanned_file::record_for_path;

/// Streams the folder's records as `scan:*` events. `on_records` gets each batch right
/// after its event, so later steps can't answer for a file the webview hasn't seen.
pub(crate) fn scan_folder(
    sink: &impl EventSink,
    root: &Path,
    cancellation_token: &CancellationToken,
    scan_id: u64,
    on_records: impl FnMut(&[FileRecord]),
) {
    events::scan_started(
        sink,
        ScanStartedEvent {
            root: root.to_string_lossy().to_string(),
            scan_id,
        },
    );

    let mut totals = ScanFinishedEvent {
        discovered: 0,
        skipped: 0,
        errors: 0,
        cancelled: false,
    };
    let mut batch = RecordBatch::new(on_records);

    for entry in WalkDir::new(root) {
        if cancellation_token.is_cancelled() {
            totals.cancelled = true;
            break;
        }

        let entry = match entry {
            Ok(value) => value,
            Err(error) => {
                totals.errors += 1;
                batch.flush(sink);
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
        } else {
            totals.skipped += 1;
        }
        batch.push(sink, record);
    }

    batch.flush(sink);
    events::scan_finished(sink, totals);
}
