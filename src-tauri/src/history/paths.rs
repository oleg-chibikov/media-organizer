use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::safe_fs;

const HISTORY_DIR_NAME: &str = "history";
const HISTORY_RUNS_DIR_NAME: &str = "runs";
const HISTORY_INDEX_FILENAME: &str = "index.json";
const RUN_LOG_EXTENSION: &str = "jsonl";

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
        "{}_{}.{RUN_LOG_EXTENSION}",
        started_at.format("%Y-%m-%dT%H-%M-%SZ"),
        run_id
    )
}

/// Accepts a run log path read from the index only when it names a `.jsonl` file directly
/// in the runs folder, so a damaged index can't aim an append or delete at a user file.
pub(super) fn checked_run_log_path(history_root: &Path, recorded: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(recorded);
    let refused = |reason: String| format!("Refusing history log path '{recorded}': {reason}");
    if path.extension().and_then(|extension| extension.to_str()) != Some(RUN_LOG_EXTENSION) {
        return Err(refused("not a run log file".to_string()));
    }
    safe_fs::ensure_inside_dir(&path, &history_runs_dir(history_root))
        .map_err(|error| refused(error.to_string()))?;
    Ok(path)
}
