//! Executes a confirmed move plan: safety checks, moves, history logging and progress events.

mod history_entry;
mod lookups;
pub(crate) mod progress;
mod recorder;
mod source_cleanup;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::{
    history::ActiveHistoryRun,
    models::{FileRecord, MetadataResult, MovePlan, MoveProgressPhase, MoveResultStatus},
    mover,
};
use lookups::RunLookups;
use progress::emit_move_progress_message;
use recorder::MoveRunRecorder;

pub(crate) struct MoveRunRequest {
    pub selected_folder: PathBuf,
    pub destination_root: PathBuf,
    pub files: Vec<FileRecord>,
    pub metadata_results: Vec<MetadataResult>,
    pub plans: Vec<MovePlan>,
}

pub(crate) fn run_move_plan(
    app: &AppHandle,
    request: MoveRunRequest,
    history_run: ActiveHistoryRun,
) {
    let total = request.plans.len() as u64;
    let progress = |phase, file_id: Option<&str>, message: &str, completed| {
        emit_move_progress_message(app, phase, file_id, message, completed, total);
    };

    progress(
        MoveProgressPhase::Preparing,
        None,
        "Preparing move operations.",
        0,
    );
    let prepared = mover::prepare_move_operations(&request.files, &request.plans);
    let lookups = RunLookups::new(&request.files, &request.metadata_results, &request.plans);
    let mut recorder = MoveRunRecorder::new(app, history_run, lookups);
    let mut moved_source_directories = Vec::new();

    for result in prepared.skipped {
        recorder.record(result);
    }

    for operation in prepared.operations {
        progress(
            MoveProgressPhase::Preparing,
            Some(&operation.file_id),
            "Checking source and destination.",
            recorder.completed_operations(),
        );
        let safety_checked = mover::validate_move_safety_with_roots(
            &request.selected_folder,
            &request.destination_root,
            vec![operation],
        );
        for result in safety_checked.skipped {
            recorder.record(result);
        }

        let rechecked = mover::recheck_destination_conflicts(safety_checked.ready);
        for result in rechecked.skipped {
            recorder.record(result);
        }

        for operation in rechecked.ready {
            progress(
                MoveProgressPhase::Moving,
                Some(&operation.file_id),
                "Moving file.",
                recorder.completed_operations(),
            );
            let source_parent = Path::new(&operation.source_path)
                .parent()
                .map(Path::to_path_buf);
            let result = mover::execute_move_with_rename(&operation);
            if matches!(result.result_status, MoveResultStatus::Moved) {
                moved_source_directories.extend(source_parent);
            }
            recorder.record(result);
        }
    }

    progress(
        MoveProgressPhase::Finalizing,
        None,
        "Finalizing history and cleaning up source folders.",
        recorder.completed_operations(),
    );
    source_cleanup::cleanup_empty_source_directories(
        &request.selected_folder,
        &moved_source_directories,
    );
    recorder.finish();
}
