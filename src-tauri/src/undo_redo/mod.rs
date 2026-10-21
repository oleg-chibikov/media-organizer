//! Undo and redo of recorded history operations, validated against the current filesystem.

mod eligibility;
mod outcome;
mod records;
mod redo;
mod replay;
mod run;
mod undo;

#[cfg(test)]
mod tests;

pub(crate) use eligibility::{redoable_operations, undoable_operations};
pub(crate) use run::{run_redo, run_undo};
