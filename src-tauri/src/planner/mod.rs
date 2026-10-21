//! Destination planning: dated folders, original-name filenames and conflict suffixes.

mod content_match;
mod destination_allocation;
mod file_plan;

#[cfg(test)]
mod tests;

use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::{
    events::{self, EventSink},
    models::{
        FileKind, FileRecord, MetadataResult, PlanErrorEvent, PlanFileEvent, PlanFinishedEvent,
    },
};
use file_plan::create_plan_for_file;

/// Plans every photo/video under `destination_root`, streaming one event per file.
/// Planning is a dry run: it never creates folders or files.
pub(crate) fn stream_move_plan(
    sink: &impl EventSink,
    destination_root: &Path,
    files: &[FileRecord],
    metadata_results: &[MetadataResult],
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
        match create_plan_for_file(destination_root, file, metadata, &mut reserved_paths) {
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
