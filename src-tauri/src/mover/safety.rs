use std::path::Path;

use super::{outcome::skipped_operation, CheckedMoves, MoveOperation};
use crate::path_safety::{
    has_parent_dir_component, path_is_within_case_insensitive, paths_equal_case_insensitive,
};

/// Legacy check that keeps destinations inside `<selected_root>/Processed`.
#[cfg(test)]
pub(super) fn validate_move_safety(
    selected_root: &Path,
    operations: Vec<MoveOperation>,
) -> CheckedMoves {
    let processed_root = selected_root.join("Processed");
    validate_move_safety_internal(
        selected_root,
        &processed_root,
        operations,
        "Destination file is outside the selected Processed folder.",
    )
}

pub(crate) fn validate_move_safety_with_roots(
    selected_source_root: &Path,
    effective_destination_root: &Path,
    operations: Vec<MoveOperation>,
) -> CheckedMoves {
    validate_move_safety_internal(
        selected_source_root,
        effective_destination_root,
        operations,
        "Destination file is outside the effective destination root.",
    )
}

fn validate_move_safety_internal(
    selected_source_root: &Path,
    effective_destination_root: &Path,
    operations: Vec<MoveOperation>,
    destination_outside_root_error: &str,
) -> CheckedMoves {
    let mut checked = CheckedMoves::default();

    for operation in operations {
        let source = Path::new(&operation.source_path);
        let destination = Path::new(&operation.destination_path);

        let violation = if has_parent_dir_component(source) || has_parent_dir_component(destination)
        {
            Some("Source and destination paths must not contain parent-directory traversal.")
        } else if paths_equal_case_insensitive(source, destination) {
            Some("Source and destination resolve to the same path.")
        } else if !path_is_within_case_insensitive(source, selected_source_root) {
            Some("Source file is outside the selected folder.")
        } else if !path_is_within_case_insensitive(destination, effective_destination_root) {
            Some(destination_outside_root_error)
        } else if !source.is_file() {
            Some("Source path is not a regular file at execution time.")
        } else {
            None
        };

        match violation {
            Some(reason) => checked.skipped.push(skipped_operation(&operation, reason)),
            None => checked.ready.push(operation),
        }
    }

    checked
}
