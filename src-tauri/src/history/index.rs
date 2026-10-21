use std::{fs, path::Path};

use super::{paths::history_index_path, HISTORY_VERSION};
use crate::models::HistoryIndexEntry;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct HistoryIndexFile {
    pub history_version: u32,
    pub runs: Vec<HistoryIndexEntry>,
}

impl HistoryIndexFile {
    pub(super) fn find(&self, run_id: &str) -> Result<&HistoryIndexEntry, String> {
        self.runs
            .iter()
            .find(|entry| entry.run_id == run_id)
            .ok_or_else(|| format!("History run '{run_id}' was not found in index."))
    }
}

pub(super) fn load_history_index(history_root: &Path) -> Result<HistoryIndexFile, String> {
    let index_path = history_index_path(history_root);
    if !index_path.exists() {
        return Ok(HistoryIndexFile {
            history_version: HISTORY_VERSION,
            runs: Vec::new(),
        });
    }

    let raw = fs::read_to_string(&index_path).map_err(|error| {
        format!(
            "Failed to read history index at '{}': {error}",
            index_path.display()
        )
    })?;

    serde_json::from_str(&raw).map_err(|error| {
        format!(
            "Failed to parse history index at '{}': {error}",
            index_path.display()
        )
    })
}

pub(super) fn write_history_index(
    history_root: &Path,
    index: &HistoryIndexFile,
) -> Result<(), String> {
    fs::create_dir_all(history_root).map_err(|error| {
        format!(
            "Failed to create history directory at '{}': {error}",
            history_root.display()
        )
    })?;

    let serialized = serde_json::to_string_pretty(index)
        .map_err(|error| format!("Failed to serialize history index: {error}"))?;

    let index_path = history_index_path(history_root);
    fs::write(&index_path, serialized).map_err(|error| {
        format!(
            "Failed to write history index at '{}': {error}",
            index_path.display()
        )
    })
}
