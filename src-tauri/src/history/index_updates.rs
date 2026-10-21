use std::path::Path;

use super::{
    index::{load_history_index, with_index_lock, write_history_index},
    MAX_INDEX_RUNS,
};
use crate::models::{HistoryIndexEntry, HistoryRun};

pub(super) fn upsert_history_index_entry(
    history_root: &Path,
    entry: HistoryIndexEntry,
) -> Result<(), String> {
    with_index_lock(|| {
        let mut index = load_history_index(history_root)?;
        if let Some(existing) = index.runs.iter_mut().find(|run| run.run_id == entry.run_id) {
            *existing = entry;
        } else {
            index.runs.push(entry);
        }

        keep_newest_runs(&mut index.runs);
        write_history_index(history_root, &index)
    })
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
