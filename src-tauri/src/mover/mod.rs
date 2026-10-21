//! Move preparation, root safety validation, conflict recheck and no-overwrite execution.

mod confirmation_summary;
mod conflict_recheck;
mod copy_fallback;
mod outcome;
mod preparation;
mod rename;
mod safety;

#[cfg(test)]
mod tests;

pub(crate) use confirmation_summary::build_move_confirmation_summary;
pub(crate) use conflict_recheck::recheck_destination_conflicts;
pub(crate) use preparation::prepare_move_operations;
pub(crate) use rename::execute_move_with_rename;
pub(crate) use safety::validate_move_safety_with_roots;

use crate::models::MoveResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveOperation {
    pub file_id: String,
    pub source_path: String,
    pub destination_path: String,
}

/// Operations that can proceed, plus skip results for everything that cannot.
#[derive(Debug, Clone, Default)]
pub struct CheckedMoves {
    pub ready: Vec<MoveOperation>,
    pub skipped: Vec<MoveResult>,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedMoves {
    pub operations: Vec<MoveOperation>,
    pub skipped: Vec<MoveResult>,
}
