use super::{
    eligibility::{redoable_operations, undoable_operations},
    outcome::OperationOutcome,
    records::{build_redo_history_operation, build_undo_history_operation},
    redo::execute_redo_operation,
    undo::execute_undo_operation,
};
use crate::{
    events::{self, EventSink},
    models::{HistoryOperation, HistoryReplayProgressEvent, HistoryRunDetails, HistoryRunStatus},
};

/// Whether a replay moves files back to their sources or out to their destinations again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReplayDirection {
    Undo,
    Redo,
}

impl ReplayDirection {
    pub(super) fn eligible_operations(
        self,
        details: &HistoryRunDetails,
    ) -> Result<Vec<HistoryOperation>, String> {
        match self {
            Self::Undo => undoable_operations(details),
            Self::Redo => redoable_operations(details),
        }
    }

    pub(super) fn execute(self, operation: &HistoryOperation) -> OperationOutcome {
        match self {
            Self::Undo => execute_undo_operation(operation),
            Self::Redo => execute_redo_operation(operation),
        }
    }

    pub(super) fn record(
        self,
        run_id: &str,
        operation: &HistoryOperation,
        outcome: &OperationOutcome,
    ) -> HistoryOperation {
        match self {
            Self::Undo => build_undo_history_operation(run_id, operation, outcome),
            Self::Redo => build_redo_history_operation(run_id, operation, outcome),
        }
    }

    pub(super) fn fully_replayed_status(self) -> HistoryRunStatus {
        match self {
            Self::Undo => HistoryRunStatus::Undone,
            Self::Redo => HistoryRunStatus::Redone,
        }
    }

    pub(super) fn emit_progress(self, sink: &impl EventSink, payload: HistoryReplayProgressEvent) {
        match self {
            Self::Undo => events::history_undo_progress(sink, payload),
            Self::Redo => events::history_redo_progress(sink, payload),
        }
    }
}
