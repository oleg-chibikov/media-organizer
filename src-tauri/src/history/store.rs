use std::path::{Path, PathBuf};

use super::{
    active_run::{start_history_run, ActiveHistoryRun},
    log_file::RunLogWriter,
    paths::history_root_from_app_data,
    queries, run_updates,
};
use crate::models::{
    EffectiveDestinationRoot, HistoryIndexEntry, HistoryRunDetails, HistoryRunStatus,
};

/// Processing history kept under `<appData>/history`: run logs plus a bounded index.
#[derive(Debug, Clone)]
pub(crate) struct HistoryStore {
    root: PathBuf,
}

impl HistoryStore {
    pub(crate) fn in_app_data(app_data_dir: &Path) -> Self {
        Self {
            root: history_root_from_app_data(app_data_dir),
        }
    }

    pub(crate) fn start_run(
        &self,
        selected_folder: &Path,
        destination_root: &EffectiveDestinationRoot,
        place_root: &Path,
    ) -> Result<ActiveHistoryRun, String> {
        start_history_run(&self.root, selected_folder, destination_root, place_root)
    }

    pub(crate) fn list_runs(&self) -> Result<Vec<HistoryIndexEntry>, String> {
        queries::list_history_runs(&self.root)
    }

    pub(crate) fn get_run(&self, run_id: &str) -> Result<Option<HistoryRunDetails>, String> {
        queries::get_history_run(&self.root, run_id)
    }

    pub(crate) fn require_run(&self, run_id: &str) -> Result<HistoryRunDetails, String> {
        self.get_run(run_id)?
            .ok_or_else(|| format!("History run '{run_id}' was not found."))
    }

    pub(crate) fn open_run_log(&self, run_id: &str) -> Result<RunLogWriter, String> {
        run_updates::open_run_log(&self.root, run_id)
    }

    pub(crate) fn update_run_status(
        &self,
        run_id: &str,
        run_status: HistoryRunStatus,
    ) -> Result<(), String> {
        run_updates::update_history_run_status(&self.root, run_id, run_status)
    }

    pub(crate) fn delete_run(&self, run_id: &str) -> Result<(), String> {
        run_updates::delete_history_run(&self.root, run_id)
    }
}
