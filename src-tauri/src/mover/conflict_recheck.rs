use std::path::Path;

use super::{outcome::skipped_operation, CheckedMoves, MoveOperation};
use crate::safe_fs::path_is_occupied;

/// Skips operations whose destination appeared after planning.
pub(crate) fn recheck_destination_conflicts(operations: Vec<MoveOperation>) -> CheckedMoves {
    let mut checked = CheckedMoves::default();

    for operation in operations {
        if path_is_occupied(Path::new(&operation.destination_path)) {
            checked.skipped.push(skipped_operation(
                &operation,
                "Destination file already exists at execution time.",
            ));
        } else {
            checked.ready.push(operation);
        }
    }

    checked
}
