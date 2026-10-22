//! Earlier days of the scan's places that move into the places' new folders.

use std::sync::PoisonError;

use super::Planning;
use crate::{events::EventSink, models::LibraryFolderMove, safe_fs, scan_session::ScanSession};

impl ScanSession {
    /// Counts the destination's folders of the scan's places from nearby days, then raises
    /// folders to where the destination has them. Files of those earlier days move too.
    /// A layout error shows on the files once they plan.
    pub(super) fn fit_destination(&self, planning: &mut Planning) {
        self.drop_library(planning);
        let Ok(layout) = self.layout(planning) else {
            return;
        };
        let Planning {
            places: Some(folders),
            reservations,
            library_apart,
            ..
        } = planning
        else {
            return;
        };
        let Some(root) = layout.place_root() else {
            return;
        };
        let nearby = if *library_apart {
            Vec::new()
        } else {
            folders.gather_nearby(root)
        };
        folders.reuse_existing(root);
        let library = folders.library_moves(root, &nearby, &self.context.folder, reservations);
        self.add_records(&library.records);
        for plan in &library.plans {
            planning.plans.insert(plan.file_id.clone(), plan.clone());
        }
        planning.library = library;
    }

    pub(super) fn drop_library(&self, planning: &mut Planning) {
        let library = std::mem::take(&mut planning.library);
        let mut records = self.records.write().unwrap_or_else(PoisonError::into_inner);
        for plan in &library.plans {
            planning.plans.remove(&plan.file_id);
            planning.reservations.release(&plan.file_id);
            records.remove(&plan.file_id);
        }
    }

    /// Leaves earlier days where they are, or gathers them again, and plans every file anew.
    pub(crate) fn set_gather_library(&self, sink: &impl EventSink, gather: bool) {
        let mut planning = self.planning();
        planning.library_apart = !gather;
        let applied = planning.file_places.clone();
        let preview = planning.preview;
        drop(planning);
        if let Some(file_places) = applied {
            self.set_places(sink, file_places, preview);
        }
    }

    /// Whether earlier days are gathered, and the folders they come from.
    pub(crate) fn library_folders(&self) -> (bool, Vec<LibraryFolderMove>) {
        let planning = self.planning();
        (!planning.library_apart, planning.library.folders.clone())
    }

    /// Removes the folders of earlier days the run emptied, and their emptied parents.
    pub(in crate::scan_session) fn remove_emptied_library_folders(&self) {
        let planning = self.planning();
        let Some(root) = planning
            .layout
            .as_ref()
            .and_then(|layout| layout.place_root())
        else {
            return;
        };
        for folder in &planning.library.folders {
            let from = std::path::Path::new(&folder.from);
            safe_fs::remove_empty_dir_within(from, root, &self.context.folder);
        }
    }
}
