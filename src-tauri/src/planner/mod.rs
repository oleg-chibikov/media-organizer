//! Destination planning: dated, place or same folders, original-name filenames and conflict suffixes.

mod continued_moves;
mod continued_spans;
mod country_scope;
mod destination_allocation;
mod destination_layout;
mod file_plan;
mod place_dates;
mod place_folder;
mod place_folders;
mod placed_file;
mod placed_moves;
mod suggestion_folders;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use crate::{
    events::{self, EventSink},
    models::{
        FileKind, FileRecord, MetadataResult, PlanErrorEvent, PlanFileEvent, PlanFinishedEvent,
    },
};
pub(crate) use continued_moves::plan_continued_moves;
pub(crate) use destination_layout::DestinationLayout;
use file_plan::create_plan_for_file;
pub(crate) use place_folders::PlaceFolders;

/// Plans every photo/video into `layout`, streaming one event per file. Files
/// in `place_folders` go to their place folder instead of a date folder.
/// Planning is a dry run: it doesn't create folders or files.
pub(crate) fn stream_move_plan(
    sink: &impl EventSink,
    layout: &DestinationLayout,
    files: &[FileRecord],
    metadata_results: &[MetadataResult],
    place_folders: &PlaceFolders,
) {
    let metadata_by_id: HashMap<&str, &MetadataResult> = metadata_results
        .iter()
        .map(|result| (result.file_id.as_str(), result))
        .collect();
    let mut reserved_paths: HashSet<String> = HashSet::new();
    let mut planned = 0_u64;
    let mut errors = 0_u64;

    for file in files {
        if !matches!(file.kind, FileKind::Photo | FileKind::Video) {
            continue;
        }

        let metadata = metadata_by_id.get(file.id.as_str()).copied();
        match create_plan_for_file(
            layout,
            file,
            metadata,
            place_folders.get(&file.id),
            &mut reserved_paths,
        ) {
            Ok(plan) => {
                planned += 1;
                events::plan_file(sink, PlanFileEvent { plan });
            }
            Err(error) => {
                errors += 1;
                events::plan_error(
                    sink,
                    PlanErrorEvent {
                        file_id: file.id.clone(),
                        error,
                    },
                );
            }
        }
    }

    events::plan_finished(sink, PlanFinishedEvent { planned, errors });
}
