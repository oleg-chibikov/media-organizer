use std::{fs, path::Path};

use super::{
    copy_io::reserve_temp_destination,
    outcome::{attempted_result, skipped_operation},
    verification::verify_destination_matches_source,
    MoveOperation,
};
use crate::{
    file_ops::{copy_into, copy_into_new_file},
    models::{MoveResult, MoveResultStatus, MoveStrategy},
};

/// Copies through a reserved temp file, verifies bytes, then publishes without overwrite.
/// The source is always kept.
pub(super) fn execute_copy_verify_keep_source(
    operation: &MoveOperation,
    source: &Path,
    destination: &Path,
) -> MoveResult {
    let failed = |error: String| {
        attempted_result(
            operation,
            MoveStrategy::CopyKeepSource,
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

    let (temp_destination, mut temp_file) = match reserve_temp_destination(destination) {
        Ok(reserved) => reserved,
        Err(error) => return failed(format!("Failed to reserve temporary copy path: {error}")),
    };

    if let Err(error) = copy_into(source, &mut temp_file) {
        drop(temp_file);
        let _ = fs::remove_file(&temp_destination);
        return failed(format!("Copy fallback failed: {error}"));
    }

    drop(temp_file);

    if let Err(error) = verify_destination_matches_source(source, &temp_destination) {
        let _ = fs::remove_file(&temp_destination);
        return failed(error);
    }

    if destination.exists() {
        let _ = fs::remove_file(&temp_destination);
        return skipped_operation(
            operation,
            "Destination appeared during copy fallback. Refusing to overwrite.",
        );
    }

    if let Err(error) = copy_into_new_file(&temp_destination, destination) {
        let _ = fs::remove_file(&temp_destination);
        return failed(format!("Failed to persist verified copy fallback: {error}"));
    }

    let _ = fs::remove_file(&temp_destination);

    if let Err(error) = verify_destination_matches_source(source, destination) {
        return failed(format!(
            "{error} Source file was kept; destination copy should be inspected."
        ));
    }

    attempted_result(
        operation,
        MoveStrategy::CopyKeepSource,
        MoveResultStatus::Copied,
        None,
    )
}
