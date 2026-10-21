use std::{io, path::Path};

use super::{
    outcome::{attempted_result, skipped_operation},
    MoveOperation,
};
use crate::{
    models::{MoveResult, MoveResultStatus, MoveStrategy},
    safe_fs,
};

/// Publishes a verified copy without overwriting anything. The source is always kept.
pub(super) fn execute_copy_verify_keep_source(
    operation: &MoveOperation,
    source: &Path,
    destination: &Path,
) -> MoveResult {
    match safe_fs::copy_verified_no_replace(source, destination) {
        Ok(()) => attempted_result(
            operation,
            MoveStrategy::CopyKeepSource,
            MoveResultStatus::Copied,
            None,
        ),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => skipped_operation(
            operation,
            "Destination appeared during copy fallback. Refusing to overwrite.",
        ),
        Err(error) => attempted_result(
            operation,
            MoveStrategy::CopyKeepSource,
            MoveResultStatus::Failed,
            Some(format!(
                "Copy fallback failed; the source file was kept: {error}"
            )),
        ),
    }
}
