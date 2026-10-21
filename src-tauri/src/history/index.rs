use std::{fs, path::Path};

use super::{paths::history_index_path, HISTORY_VERSION, MAX_INDEX_RUNS};
use crate::models::{HistoryIndexEntry, HistoryRun};

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

pub(super) fn upsert_history_index_entry(
    history_root: &Path,
    entry: HistoryIndexEntry,
) -> Result<(), String> {
    let mut index = load_history_index(history_root)?;

    if let Some(existing) = index.runs.iter_mut().find(|run| run.run_id == entry.run_id) {
        *existing = entry;
    } else {
        index.runs.push(entry);
    }

    keep_newest_runs(&mut index.runs);
    write_history_index(history_root, &index)
}

/// Sorts newest first and drops runs beyond the index capacity.
pub(super) fn keep_newest_runs(runs: &mut Vec<HistoryIndexEntry>) {
    runs.sort_by(|left, right| {
        right
            .started_at
            .cmp(&left.started_at)
            .then_with(|| right.run_id.cmp(&left.run_id))
    });
    runs.truncate(MAX_INDEX_RUNS);
}

pub(super) fn index_entry_from_run(run: &HistoryRun) -> HistoryIndexEntry {
    HistoryIndexEntry {
        run_id: run.run_id.clone(),
        folder_label: run.folder_label.clone(),
        selected_folder: run.selected_folder.clone(),
        destination_root_path: run.destination_root_path.clone(),
        started_at: run.started_at.clone(),
        completed_at: run.completed_at.clone(),
        run_status: run.run_status.clone(),
        files_migrated: 0,
        undo_available: false,
        redo_available: false,
        history_file_path: run.history_file_path.clone(),
    }
}
