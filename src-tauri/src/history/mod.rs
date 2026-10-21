//! Processing history: appData JSONL run logs, a bounded run index and crash recovery.

mod active_run;
mod candidates;
mod ids;
mod index;
mod index_updates;
mod log_file;
mod log_lines;
mod log_parser;
mod parsed_run_log;
mod paths;
mod queries;
mod recovery;
mod run_record;
mod run_updates;

#[cfg(test)]
mod tests;

pub const HISTORY_VERSION: u32 = 1;
pub const MAX_INDEX_RUNS: usize = 200;

pub(crate) use active_run::{
    append_history_operation, complete_history_run, start_history_run_with_destination_root,
    ActiveHistoryRun,
};
pub(crate) use candidates::{collect_redoable_operations, collect_undoable_operations};
pub(crate) use ids::next_operation_id;
pub(crate) use queries::{get_history_run, list_history_runs};
pub(crate) use run_updates::{
    append_history_operation_for_run, delete_history_run, update_history_run_status,
};
