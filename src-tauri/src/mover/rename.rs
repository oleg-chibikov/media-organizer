use std::{fs, io, path::Path};

use super::{
    copy_fallback::execute_copy_verify_keep_source,
    outcome::{attempted_result, skipped_operation},
    MoveOperation,
};
use crate::models::{MoveResult, MoveResultStatus, MoveStrategy};

const EEXIST: i32 = 17;

pub(crate) fn execute_move_with_rename(operation: &MoveOperation) -> MoveResult {
    execute_move_with_rename_internal(operation, |source, destination| {
        fs::rename(source, destination)
    })
}

/// Renames without overwriting; cross-device failures fall back to a verified copy.
pub(super) fn execute_move_with_rename_internal<F>(
    operation: &MoveOperation,
    rename_fn: F,
) -> MoveResult
where
    F: Fn(&Path, &Path) -> io::Result<()>,
{
    let source = Path::new(&operation.source_path);
    let destination = Path::new(&operation.destination_path);
    let failed = |error: String| {
        attempted_result(
            operation,
            MoveStrategy::Rename,
            MoveResultStatus::Failed,
            Some(error),
        )
    };

    if destination.exists() {
        return skipped_operation(
            operation,
            "Destination already exists. Refusing to overwrite.",
        );
    }

    if !source.exists() {
        return failed("Source file no longer exists at execution time.".to_string());
    }

    if let Some(parent) = destination.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            return failed(format!(
                "Failed to create destination directory before rename: {error}"
            ));
        }
    }

    match rename_fn(source, destination) {
        Ok(()) => attempted_result(
            operation,
            MoveStrategy::Rename,
            MoveResultStatus::Moved,
            None,
        ),
        Err(error) if should_fallback_to_copy_keep_source(&error) => {
            execute_copy_verify_keep_source(operation, source, destination)
        }
        Err(error) => failed(format!("Atomic rename failed: {error}")),
    }
}

fn should_fallback_to_copy_keep_source(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::CrossesDevices || error.raw_os_error() == Some(EEXIST)
}
