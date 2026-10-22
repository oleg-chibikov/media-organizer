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
    placements::PlacementStore,
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
    placements: &PlacementStore,
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
    let history_run = history.start_run(selected_folder, &effective_root)?;
    events::history_index_updated(sink, &history_run.run.run_id);

    Ok(StartedMoveRun {
        request: MoveRunRequest {
            selected_folder: selected_folder.to_path_buf(),
            destination_root: PathBuf::from(&effective_root.root_path),
            batch,
            placements: placements.clone(),
        },
        history_run,
    })
}

impl StartedMoveRun {
    /// Moves every ready file until `pause` is cancelled; blocks until the run is recorded.
    pub(crate) fn execute(self, sink: &impl EventSink, pause: &CancellationToken) {
        run_move_plan(sink, self.request, self.history_run, pause);
    }
}
