use std::collections::HashMap;

use tauri::AppHandle;

use super::{
    history_entry::{
        build_history_operation_for_result, ensure_expected_source_size,
        history_run_status_for_completion,
    },
    lookups::RunLookups,
};
use crate::{
    events,
    history::{self, ActiveHistoryRun},
    models::{
        HistoryIndexUpdatedEvent, MoveFileEvent, MoveFinishedEvent, MoveResult, MoveResultStatus,
    },
};

/// Logs every move result to history, tallies it and streams it to the frontend.
pub(super) struct MoveRunRecorder<'a> {
    app: &'a AppHandle,
    history_run: ActiveHistoryRun,
    lookups: RunLookups<'a>,
    expected_source_sizes: HashMap<&'a str, Option<u64>>,
    moved: u64,
    copied: u64,
    skipped: u64,
    failed: u64,
    results: Vec<MoveResult>,
    history_log_had_error: bool,
}

impl<'a> MoveRunRecorder<'a> {
    pub(super) fn new(
        app: &'a AppHandle,
        history_run: ActiveHistoryRun,
        lookups: RunLookups<'a>,
    ) -> Self {
        Self {
            app,
            history_run,
            lookups,
            expected_source_sizes: HashMap::new(),
            moved: 0,
            copied: 0,
            skipped: 0,
            failed: 0,
            results: Vec::new(),
            history_log_had_error: false,
        }
    }

    pub(super) fn completed_operations(&self) -> u64 {
        self.results.len() as u64
    }

    pub(super) fn record(&mut self, result: MoveResult) {
        ensure_expected_source_size(
            &result,
            &self.lookups.files_by_id,
            &mut self.expected_source_sizes,
        );
        let operation = build_history_operation_for_result(
            &self.history_run.run.run_id,
            &result,
            &self.lookups,
            &self.expected_source_sizes,
        );
        if history::append_history_operation(&mut self.history_run, operation).is_err() {
            self.history_log_had_error = true;
        }

        match result.result_status {
            MoveResultStatus::Moved => self.moved += 1,
            MoveResultStatus::Copied => self.copied += 1,
            MoveResultStatus::Skipped => self.skipped += 1,
            MoveResultStatus::Failed => self.failed += 1,
        }
        self.results.push(result.clone());
        let _ = events::emit_move_file(self.app, MoveFileEvent { result });
    }

    pub(super) fn finish(mut self) {
        let run_status = history_run_status_for_completion(self.failed, self.history_log_had_error);
        let history_run_id = self.history_run.run.run_id.clone();
        let _ = history::complete_history_run(&mut self.history_run, run_status);
        let _ = events::emit_history_index_updated(
            self.app,
            HistoryIndexUpdatedEvent {
                run_id: Some(history_run_id),
            },
        );

        let _ = events::emit_move_finished(
            self.app,
            MoveFinishedEvent {
                moved: self.moved,
                copied: self.copied,
                skipped: self.skipped,
                failed: self.failed,
                results: self.results,
            },
        );
    }
}
