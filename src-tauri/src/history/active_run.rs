use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;

use super::{
    ids::generate_run_id,
    index_updates::{index_entry_from_run, upsert_history_index_entry},
    log_file::RunLogWriter,
    paths::{history_runs_dir, run_log_filename},
    run_record::new_history_run,
};
use crate::models::{
    EffectiveDestinationRoot, HistoryOperation, HistoryRecord, HistoryRun,
    HistoryRunCompletedRecord, HistoryRunStatus,
};

/// A processing run whose log file stays open while moves execute.
#[derive(Debug)]
pub(crate) struct ActiveHistoryRun {
    pub run: HistoryRun,
    history_root: PathBuf,
    log: RunLogWriter,
}

pub(super) fn start_history_run(
    history_root: &Path,
    selected_folder: &Path,
    destination_root: &EffectiveDestinationRoot,
) -> Result<ActiveHistoryRun, String> {
    let runs_dir = history_runs_dir(history_root);
    fs::create_dir_all(&runs_dir).map_err(|error| {
        format!(
            "Failed to create history runs directory at '{}': {error}",
            runs_dir.display()
        )
    })?;

    let started_at = Utc::now();
    let run_id = generate_run_id(started_at);
    let log_path = runs_dir.join(run_log_filename(started_at, &run_id));
    let mut log = RunLogWriter::create(&log_path)?;
    let run = new_history_run(
        run_id,
        selected_folder,
        destination_root,
        started_at,
        &log_path,
    );

    log.append(&HistoryRecord::RunStarted { run: run.clone() })?;
    upsert_history_index_entry(history_root, index_entry_from_run(&run))?;

    Ok(ActiveHistoryRun {
        run,
        history_root: history_root.to_path_buf(),
        log,
    })
}

impl ActiveHistoryRun {
    pub(crate) fn append_operation(&mut self, operation: HistoryOperation) -> Result<(), String> {
        self.log.append_operation(operation)
    }

    #[cfg(test)]
    pub(crate) fn break_log_for_tests(&mut self) {
        self.log = RunLogWriter::read_only_for_tests(Path::new(&self.run.history_file_path));
    }

    pub(crate) fn complete(&mut self, run_status: HistoryRunStatus) -> Result<(), String> {
        let completed_at = Utc::now().to_rfc3339();
        self.log
            .append(&HistoryRecord::RunCompleted(HistoryRunCompletedRecord {
                run_id: self.run.run_id.clone(),
                completed_at: completed_at.clone(),
                run_status: run_status.clone(),
            }))?;

        self.run.completed_at = Some(completed_at);
        self.run.run_status = run_status;
        upsert_history_index_entry(&self.history_root, index_entry_from_run(&self.run))
    }
}
