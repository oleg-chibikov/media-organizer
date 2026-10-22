//! Executes a confirmed move plan: safety checks, moves, history logging and progress events.

mod execution;
mod file_step;
mod history_entry;
mod placement_record;
mod progress;
mod recorder;
mod request;
mod source_cleanup;
mod start;
mod status_mapping;

#[cfg(test)]
mod tests;

pub(crate) use request::MoveBatch;
pub(crate) use source_cleanup::remove_empty_source_folders;
pub(crate) use start::start_move_run;
