use std::path::PathBuf;

use super::direction::ReplayDirection;
use crate::{
    models::{HistoryOperation, HistoryOperationStatus},
    placements::{restore_files, PlacementStore, UndoneMove},
};

/// Collects the files an undo took out of the destination root, then updates the placed
/// suggestions: placed files that went back to a place folder are recorded there again,
/// the rest are dropped. Redo doesn't record them again.
pub(super) struct UndonePlacements {
    store: PlacementStore,
    destination_root: Option<PathBuf>,
    undone: Vec<UndoneMove>,
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
            self.undone.push(UndoneMove {
                source: PathBuf::from(&process.source_path),
                destination: PathBuf::from(&process.destination_path),
            });
        }
    }

    pub(super) fn forget(self) {
        let Some(root) = self.destination_root else {
            return;
        };
        if self.undone.is_empty() {
            return;
        }
        if let Err(error) = self
            .store
            .update(|records| restore_files(records, &root, &self.undone))
        {
            log::warn!("Failed to update placed suggestions after an undo: {error}");
        }
    }
}
