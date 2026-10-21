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
mod store;

#[cfg(test)]
mod tests;

pub const HISTORY_VERSION: u32 = 1;
pub const MAX_INDEX_RUNS: usize = 200;

pub(crate) use active_run::ActiveHistoryRun;
pub(crate) use candidates::{collect_redoable_operations, collect_undoable_operations};
pub(crate) use ids::next_operation_id;
pub(crate) use log_file::RunLogWriter;
pub(crate) use store::HistoryStore;
