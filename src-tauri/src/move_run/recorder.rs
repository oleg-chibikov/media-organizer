use std::collections::HashMap;

use super::{
    history_entry::{build_history_operation_for_result, ensure_expected_source_size},
    request::RunLookups,
    status_mapping::history_run_status_for_completion,
};
use crate::{
    events::{self, EventSink},
    history::ActiveHistoryRun,
    models::{MoveFileEvent, MoveFinishedEvent, MoveResult, MoveResultStatus},
};

/// Logs every move result to history, tallies it and streams it to the frontend.
pub(super) struct MoveRunRecorder<'a, S> {
    sink: &'a S,
    history_run: ActiveHistoryRun,
    lookups: RunLookups<'a>,
    expected_source_sizes: HashMap<&'a str, Option<u64>>,
    finished: MoveFinishedEvent,
    history_log_had_error: bool,
}

impl<'a, S: EventSink> MoveRunRecorder<'a, S> {
    pub(super) fn new(sink: &'a S, history_run: ActiveHistoryRun, lookups: RunLookups<'a>) -> Self {
        Self {
            sink,
            history_run,
            lookups,
            expected_source_sizes: HashMap::new(),
            finished: MoveFinishedEvent {
                moved: 0,
                copied: 0,
                skipped: 0,
                failed: 0,
                results: Vec::new(),
            },
            history_log_had_error: false,
        }
    }

    pub(super) fn completed_operations(&self) -> u64 {
        self.finished.results.len() as u64
    }

    pub(super) fn record_all(&mut self, results: Vec<MoveResult>) {
        for result in results {
            self.record(result);
        }
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
        if self.history_run.append_operation(operation).is_err() {
            self.history_log_had_error = true;
        }

        match result.result_status {
            MoveResultStatus::Moved => self.finished.moved += 1,
            MoveResultStatus::Copied => self.finished.copied += 1,
            MoveResultStatus::Skipped => self.finished.skipped += 1,
            MoveResultStatus::Failed => self.finished.failed += 1,
        }
        self.finished.results.push(result.clone());
        events::move_file(self.sink, MoveFileEvent { result });
    }

    pub(super) fn finish(mut self) {
        let run_status =
            history_run_status_for_completion(self.finished.failed, self.history_log_had_error);
        // A failed completion record leaves the run incomplete, which recovery reports as partial.
        let _ = self.history_run.complete(run_status);
        events::history_index_updated(self.sink, &self.history_run.run.run_id);
        events::move_finished(self.sink, self.finished);
    }
}
