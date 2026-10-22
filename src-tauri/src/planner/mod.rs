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

use crate::{
    events::{self, EventSink},
    models::{
        FileKind, FileRecord, MetadataResult, MovePlan, PlanErrorEvent, PlanFileEvent,
        PlanFinishedEvent,
    },
};
pub(crate) use destination_layout::DestinationLayout;
use file_plan::create_plan_for_file;
pub(crate) use place_folders::PlaceFolders;
pub(crate) use reservations::Reservations;

/// Plans every photo/video into `layout`, streaming one event per file tagged with
/// `scan_id`. Files in `place_folders` go to their place folder instead of a date folder.
/// `reservations` holds the names the scan's earlier plans took, so no two files get one.
/// Planning is a dry run: it doesn't create folders or files. Returns the plans it made.
pub(crate) fn stream_move_plan(
    sink: &impl EventSink,
    scan_id: u64,
    layout: &DestinationLayout,
    files: &[FileRecord],
    metadata_results: &[MetadataResult],
    place_folders: &PlaceFolders,
    reservations: &mut Reservations,
) -> Vec<MovePlan> {
    let metadata_by_id: HashMap<&str, &MetadataResult> = metadata_results
        .iter()
        .map(|result| (result.file_id.as_str(), result))
        .collect();
    let mut plans = Vec::new();
    let mut errors = 0_u64;

    for file in files {
        if !matches!(file.kind, FileKind::Photo | FileKind::Video) {
            continue;
        }

        reservations.release(&file.id);
        let metadata = metadata_by_id.get(file.id.as_str()).copied();
        match create_plan_for_file(layout, file, metadata, place_folders, reservations.paths()) {
            Ok(plan) => {
                reservations.hold(&plan);
                events::plan_file(
                    sink,
                    PlanFileEvent {
                        scan_id,
                        plan: plan.clone(),
                    },
                );
                plans.push(plan);
            }
            Err(error) => {
                errors += 1;
                events::plan_error(
                    sink,
                    PlanErrorEvent {
                        scan_id,
                        file_id: file.id.clone(),
                        error,
                    },
                );
            }
        }
    }

    events::plan_finished(
        sink,
        PlanFinishedEvent {
            scan_id,
            planned: plans.len() as u64,
            errors,
        },
    );
    plans
}
