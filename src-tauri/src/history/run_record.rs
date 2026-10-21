use std::path::Path;

use chrono::{DateTime, Utc};

use super::HISTORY_VERSION;
use crate::models::{EffectiveDestinationRoot, HistoryRun, HistoryRunStatus};

/// Builds the `run_started` record for a new processing run.
pub(super) fn new_history_run(
    run_id: String,
    selected_folder: &Path,
    destination_root: Option<&EffectiveDestinationRoot>,
    started_at: DateTime<Utc>,
    log_path: &Path,
) -> HistoryRun {
    HistoryRun {
        run_id,
        history_version: HISTORY_VERSION,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        selected_folder: selected_folder.to_string_lossy().to_string(),
        folder_label: folder_label_from_path(selected_folder),
        destination_root_mode: destination_root.map(|root| root.mode.clone()),
        destination_root_path: destination_root.map(|root| root.root_path.clone()),
        started_at: started_at.to_rfc3339(),
        completed_at: None,
        run_status: HistoryRunStatus::Running,
        history_file_path: log_path.to_string_lossy().to_string(),
    }
}

fn folder_label_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|segment| segment.to_str())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}
