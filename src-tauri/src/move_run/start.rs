use std::path::{Path, PathBuf};

use tokio_util::sync::CancellationToken;

use super::{
    execution::run_move_plan,
    progress::emit_move_progress_message,
    request::{MoveBatch, MoveRunRequest},
};
use crate::{
    events::{self, EventSink},
    history::{ActiveHistoryRun, HistoryStore},
    models::MoveProgressPhase,
    path_safety,
    settings::SettingsStore,
};

/// A validated batch whose history run is already open and indexed.
pub(crate) struct StartedMoveRun {
    request: MoveRunRequest,
    history_run: ActiveHistoryRun,
}

/// Validates the selected folder, resolves the destination root and opens the history run.
pub(crate) fn start_move_run(
    sink: &impl EventSink,
    settings: &SettingsStore,
    history: &HistoryStore,
    selected_folder: &Path,
    batch: MoveBatch,
) -> Result<StartedMoveRun, String> {
    path_safety::validate_selected_folder(selected_folder)?;
    emit_move_progress_message(
        sink,
        MoveProgressPhase::Starting,
        None,
        "Starting move execution.",
        0,
        batch.plans.len() as u64,
    );
    let effective_root = settings.effective_destination_root(selected_folder)?;
    let destination_root = PathBuf::from(&effective_root.root_path);
    let place_root = place_root(settings, selected_folder, &batch, &destination_root)?;
    let history_run = history.start_run(selected_folder, &effective_root, &place_root)?;
    events::history_index_updated(sink, &history_run.run.run_id);

    Ok(StartedMoveRun {
        request: MoveRunRequest {
            selected_folder: selected_folder.to_path_buf(),
            destination_root,
            place_root,
            batch,
        },
        history_run,
    })
}

/// Resolved only for a batch with files in place folders, so a missing custom root doesn't block
/// plain renames in place.
fn place_root(
    settings: &SettingsStore,
    selected_folder: &Path,
    batch: &MoveBatch,
    destination_root: &Path,
) -> Result<PathBuf, String> {
    if batch.plans.iter().any(|plan| plan.place.is_some()) {
        settings.place_folder_root(selected_folder)
    } else {
        Ok(destination_root.to_path_buf())
    }
}

impl StartedMoveRun {
    /// Moves every ready file until `pause` is cancelled; blocks until the run is recorded.
    /// Returns the ids of the files that got a result.
    pub(crate) fn execute(self, sink: &impl EventSink, pause: &CancellationToken) -> Vec<String> {
        run_move_plan(sink, self.request, self.history_run, pause)
    }
}
