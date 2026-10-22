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
    /// Where place folders went, when it differs from the destination root.
    place_root: Option<PathBuf>,
}

impl RecordedRoots {
    pub(super) fn of(run: &HistoryRun) -> Self {
        Self {
            selected_folder: PathBuf::from(&run.selected_folder),
            destination_root: run.destination_root_path.as_ref().map(PathBuf::from),
            place_root: run.place_root_path.as_ref().map(PathBuf::from),
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
        } else if !self.holds_original(original) {
            Some("recorded source is outside the run's folders")
        } else if self.destination_root.is_some() && !self.in_destination_roots(processed) {
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

    /// A source sits in the selected folder, or in the destination root when the run
    /// moved a placed file to a new day folder.
    fn holds_original(&self, original: &Path) -> bool {
        path_is_within_case_insensitive(original, &self.selected_folder)
            || self.in_destination_roots(original)
    }

    fn in_destination_roots(&self, path: &Path) -> bool {
        [&self.destination_root, &self.place_root]
            .into_iter()
            .flatten()
            .any(|root| path_is_within_case_insensitive(path, root))
    }
}
