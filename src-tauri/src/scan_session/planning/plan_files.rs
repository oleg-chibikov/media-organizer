use std::collections::HashMap;

use super::{signature::plan_signature, Planning};
use crate::{
    events::{self, EventSink},
    models::{FileKind, FileRecord, MetadataResult, PlanErrorEvent},
    path_safety,
    planner::{self, placed_files_under, DestinationLayout, PlaceFolders},
    scan_session::ScanSession,
};

/// The files to plan, with the signature each one had before.
struct Candidates {
    files: Vec<FileRecord>,
    metadata: Vec<MetadataResult>,
    previous: HashMap<String, Option<String>>,
}

impl ScanSession {
    /// Plans the files among `file_ids` whose date or place folder changed since their plan.
    pub(super) fn plan_files(
        &self,
        sink: &impl EventSink,
        planning: &mut Planning,
        file_ids: &[String],
    ) {
        let candidates = self.candidates(planning, file_ids);
        if candidates.files.is_empty() {
            return;
        }
        for file in &candidates.files {
            planning.plans.remove(&file.id);
        }
        let places = planning.places.as_deref().unwrap_or_default();
        match self.plan_inputs(places, &candidates.files) {
            Ok((layout, folders)) => {
                let plans = planner::stream_move_plan(
                    sink,
                    self.scan_id,
                    &layout,
                    &candidates.files,
                    &candidates.metadata,
                    &folders,
                    &mut planning.reservations,
                );
                for plan in plans {
                    planning.plans.insert(plan.file_id.clone(), plan);
                }
            }
            Err(error) => self.fail(sink, planning, candidates, &error),
        }
    }

    fn candidates(&self, planning: &mut Planning, file_ids: &[String]) -> Candidates {
        let mut candidates = Candidates {
            files: Vec::new(),
            metadata: Vec::new(),
            previous: HashMap::new(),
        };
        for file in self.known_records(file_ids) {
            let media = matches!(file.kind, FileKind::Photo | FileKind::Video);
            let Some(metadata) = self.metadata_of(&file.id).filter(|_| media) else {
                continue;
            };
            if planning.placed_ids.contains(&file.id) {
                continue;
            }
            let signature = plan_signature(&metadata, planning.place_keys.get(&file.id));
            let previous = planning
                .signatures
                .insert(file.id.clone(), signature.clone());
            if previous.as_ref() == Some(&signature) {
                continue;
            }
            candidates.previous.insert(file.id.clone(), previous);
            candidates.files.push(file);
            candidates.metadata.push(metadata);
        }
        candidates
    }

    fn plan_inputs(
        &self,
        places: &[crate::models::PlaceSuggestion],
        files: &[FileRecord],
    ) -> Result<(DestinationLayout, PlaceFolders), String> {
        let context = &self.context;
        path_safety::validate_selected_folder(&context.folder)?;
        let with_places = !places.is_empty();
        let layout =
            DestinationLayout::for_folder(&context.settings, &context.folder, with_places)?;
        // Day layouts come from the dates of every file in a place folder.
        let dates = if with_places {
            self.all_metadata()
        } else {
            Vec::new()
        };
        let placed = placed_files_under(
            &context.settings,
            &context.placements,
            &context.folder,
            files,
        );
        let folders = PlaceFolders::new(places, &dates)?.with_placed(placed);
        Ok((layout, folders))
    }

    /// Each file shows the error, and plans again on the next try.
    fn fail(
        &self,
        sink: &impl EventSink,
        planning: &mut Planning,
        failed: Candidates,
        error: &str,
    ) {
        for (file_id, previous) in failed.previous {
            match previous {
                Some(signature) => planning.signatures.insert(file_id.clone(), signature),
                None => planning.signatures.remove(&file_id),
            };
            let scan_id = self.scan_id;
            let error = error.to_string();
            events::plan_error(
                sink,
                PlanErrorEvent {
                    scan_id,
                    file_id,
                    error,
                },
            );
        }
    }
}
