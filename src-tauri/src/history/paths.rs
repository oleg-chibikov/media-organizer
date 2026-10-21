use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use tauri::{AppHandle, Manager};

const HISTORY_DIR_NAME: &str = "history";
const HISTORY_RUNS_DIR_NAME: &str = "runs";
const HISTORY_INDEX_FILENAME: &str = "index.json";

pub(super) fn history_root_for_app(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Failed to resolve appData directory: {error}"))?;
    Ok(history_root_from_app_data(&app_data_dir))
}

pub(super) fn history_root_from_app_data(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(HISTORY_DIR_NAME)
}

pub(super) fn history_runs_dir(history_root: &Path) -> PathBuf {
    history_root.join(HISTORY_RUNS_DIR_NAME)
}

pub(super) fn history_index_path(history_root: &Path) -> PathBuf {
    history_root.join(HISTORY_INDEX_FILENAME)
}

/// Timestamp-prefixed so run logs sort chronologically on disk.
pub(super) fn run_log_filename(started_at: DateTime<Utc>, run_id: &str) -> String {
    format!(
        "{}_{}.jsonl",
        started_at.format("%Y-%m-%dT%H-%M-%SZ"),
        run_id
    )
}
