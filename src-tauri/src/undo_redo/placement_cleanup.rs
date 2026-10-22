use std::path::{Path, PathBuf};

use super::direction::ReplayDirection;
use crate::{
    models::{HistoryOperation, HistoryOperationStatus},
    placements::{forget_files, PlacementStore},
};

/// Collects the files an undo took out of the destination root, then drops them from
/// the placed suggestions. Redo doesn't record them again.
pub(super) struct UndonePlacements {
    store: PlacementStore,
    destination_root: Option<PathBuf>,
    undone: Vec<PathBuf>,
}

impl UndonePlacements {
    pub(super) fn new(store: &PlacementStore, destination_root: Option<&str>) -> Self {
        Self {
            store: store.clone(),
            destination_root: destination_root.map(PathBuf::from),
            undone: Vec::new(),
        }
    }

    /// `process` is the operation being replayed and `record` the logged result.
    pub(super) fn note(
        &mut self,
        direction: ReplayDirection,
        process: &HistoryOperation,
        record: &HistoryOperation,
    ) {
        if direction == ReplayDirection::Undo && record.status == HistoryOperationStatus::Succeeded
        {
            self.undone.push(PathBuf::from(&process.destination_path));
        }
    }

    pub(super) fn forget(self) {
        let Some(root) = self.destination_root else {
            return;
        };
        if self.undone.is_empty() {
            return;
        }
        let paths: Vec<&Path> = self.undone.iter().map(PathBuf::as_path).collect();
        if let Err(error) = self
            .store
            .update(|records| forget_files(records, &root, &paths))
        {
            log::warn!("Failed to drop undone files from placed suggestions: {error}");
        }
    }
}
