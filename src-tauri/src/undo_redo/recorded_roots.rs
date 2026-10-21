use std::path::{Path, PathBuf};

use super::outcome::OperationOutcome;
use crate::{
    models::{HistoryOperation, HistoryOperationKind, HistoryRun},
    path_safety::{
        has_parent_dir_component, path_is_within_case_insensitive, paths_equal_case_insensitive,
    },
};

/// The folders a run was allowed to touch, read from its own start record.
pub(super) struct RecordedRoots {
    selected_folder: PathBuf,
    destination_root: Option<PathBuf>,
}

impl RecordedRoots {
    pub(super) fn of(run: &HistoryRun) -> Self {
        Self {
            selected_folder: PathBuf::from(&run.selected_folder),
            destination_root: run.destination_root_path.as_ref().map(PathBuf::from),
        }
    }

    /// Blocks a replay step whose recorded paths left the run's folders, as a damaged or
    /// edited log could make them do. Older runs without a recorded root skip that check.
    pub(super) fn check(&self, operation: &HistoryOperation) -> Result<(), OperationOutcome> {
        let (original, processed) = match operation.operation_kind {
            HistoryOperationKind::Undo => (&operation.destination_path, &operation.source_path),
            _ => (&operation.source_path, &operation.destination_path),
        };
        let (original, processed) = (Path::new(original), Path::new(processed));

        let violation = if has_parent_dir_component(original) || has_parent_dir_component(processed)
        {
            Some("recorded paths contain parent-directory traversal")
        } else if paths_equal_case_insensitive(original, processed) {
            Some("recorded source and destination are the same path")
        } else if !path_is_within_case_insensitive(original, &self.selected_folder) {
            Some("recorded source is outside the run's selected folder")
        } else if self
            .destination_root
            .as_deref()
            .is_some_and(|root| !path_is_within_case_insensitive(processed, root))
        {
            Some("recorded destination is outside the run's destination root")
        } else {
            None
        };

        match violation {
            Some(reason) => Err(OperationOutcome::blocked(format!(
                "Replay blocked: {reason}."
            ))),
            None => Ok(()),
        }
    }
}
