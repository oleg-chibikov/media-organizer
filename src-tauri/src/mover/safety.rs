use std::path::Path;

use super::{outcome::skipped_operation, CheckedMoves, MoveOperation};
use crate::path_safety::{
    has_parent_dir_component, path_is_within_case_insensitive, paths_equal_case_insensitive,
};

/// Keeps sources inside the selected folder and destinations inside the effective root.
pub(crate) fn validate_move_safety_with_roots(
    selected_source_root: &Path,
    effective_destination_root: &Path,
    operations: Vec<MoveOperation>,
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
            Some("Destination file is outside the effective destination root.")
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
