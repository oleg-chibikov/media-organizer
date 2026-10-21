use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use chrono::Utc;
use tauri::AppHandle;

use super::{
    ids::generate_run_id,
    index::{index_entry_from_run, upsert_history_index_entry},
    log_file::{append_history_record, create_run_log},
    paths::{history_root_for_app, history_runs_dir, run_log_filename},
    HISTORY_VERSION,
};
use crate::models::{
    EffectiveDestinationRoot, HistoryOperation, HistoryRecord, HistoryRun,
    HistoryRunCompletedRecord, HistoryRunStatus,
};

/// A processing run whose log file stays open while moves execute.
#[derive(Debug)]
pub struct ActiveHistoryRun {
    pub run: HistoryRun,
    history_root: PathBuf,
    log_file: File,
}

pub(crate) fn start_history_run_with_destination_root(
    app: &AppHandle,
    selected_folder: &Path,
    destination_root: Option<&EffectiveDestinationRoot>,
) -> Result<ActiveHistoryRun, String> {
    let history_root = history_root_for_app(app)?;
    let runs_dir = history_runs_dir(&history_root);
    fs::create_dir_all(&runs_dir).map_err(|error| {
        format!(
            "Failed to create history runs directory at '{}': {error}",
            runs_dir.display()
        )
    })?;

    let started_at = Utc::now();
    let run_id = generate_run_id(started_at);
    let log_path = runs_dir.join(run_log_filename(started_at, &run_id));
    let mut log_file = create_run_log(&log_path)?;

    let run = HistoryRun {
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
    };

    append_history_record(
        &mut log_file,
        &HistoryRecord::RunStarted { run: run.clone() },
    )?;
    upsert_history_index_entry(&history_root, index_entry_from_run(&run))?;

    Ok(ActiveHistoryRun {
        run,
        history_root,
        log_file,
    })
}

pub(crate) fn complete_history_run(
    active_run: &mut ActiveHistoryRun,
    run_status: HistoryRunStatus,
) -> Result<(), String> {
    let completed_at = Utc::now().to_rfc3339();
    append_history_record(
        &mut active_run.log_file,
        &HistoryRecord::RunCompleted(HistoryRunCompletedRecord {
            run_id: active_run.run.run_id.clone(),
            completed_at: completed_at.clone(),
            run_status: run_status.clone(),
        }),
    )?;

    active_run.run.completed_at = Some(completed_at);
    active_run.run.run_status = run_status;
    upsert_history_index_entry(
        &active_run.history_root,
        index_entry_from_run(&active_run.run),
    )
}

pub(crate) fn append_history_operation(
    active_run: &mut ActiveHistoryRun,
    operation: HistoryOperation,
) -> Result<(), String> {
    append_history_record(
        &mut active_run.log_file,
        &HistoryRecord::Operation(operation),
    )
}

fn folder_label_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|segment| segment.to_str())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}
