//! Destination planning: dated, place or same folders, original-name filenames and conflict suffixes.

mod destination_allocation;
mod destination_layout;
mod file_plan;
mod place_folders;
mod place_level;
mod reservations;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use crate::models::{FileKind, FileRecord, MetadataResult, MovePlan, PlanFileError};
pub(crate) use destination_layout::DestinationLayout;
use file_plan::create_plan_for_file;
pub(crate) use place_folders::PlaceFolders;
pub(crate) use reservations::Reservations;

/// What one planning pass made: a plan or an error for each photo and video.
#[derive(Default)]
pub(crate) struct PlannedFiles {
    pub plans: Vec<MovePlan>,
    pub errors: Vec<PlanFileError>,
}

/// Plans every photo/video into `layout`. Files in `place_folders` go to their place folder
/// instead of a date folder. `reservations` holds the names the scan's earlier plans took,
/// so no two files get one. Planning is a dry run: it doesn't create folders or files.
pub(crate) fn plan_files(
    layout: &DestinationLayout,
    files: &[FileRecord],
    metadata_results: &[MetadataResult],
    place_folders: &PlaceFolders,
    reservations: &mut Reservations,
) -> PlannedFiles {
    let metadata_by_id: HashMap<&str, &MetadataResult> = metadata_results
        .iter()
        .map(|result| (result.file_id.as_str(), result))
        .collect();
    let mut planned = PlannedFiles::default();
    for file in files {
        if !matches!(file.kind, FileKind::Photo | FileKind::Video) {
            continue;
        }
        reservations.release(&file.id);
        let metadata = metadata_by_id.get(file.id.as_str()).copied();
        match create_plan_for_file(layout, file, metadata, place_folders, reservations.paths()) {
            Ok(plan) => {
                reservations.hold(&plan);
                planned.plans.push(plan);
            }
            Err(error) => planned.errors.push(PlanFileError {
                file_id: file.id.clone(),
                error,
            }),
        }
    }
    planned
}
