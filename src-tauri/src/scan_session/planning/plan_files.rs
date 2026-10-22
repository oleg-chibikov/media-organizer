use std::collections::HashMap;

use super::{signature::plan_signature, Planning};
use crate::{
    events::{self, EventSink},
    models::{FileKind, FileRecord, MetadataResult, PlanErrorEvent},
    path_safety,
    planner::{self, DestinationLayout},
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
        let Planning {
            places,
            plans,
            reservations,
            signatures,
        } = planning;
        let Some(folders) = places.as_ref() else {
            return;
        };
        match self.layout(!folders.is_empty()) {
            Ok(layout) => {
                let planned = planner::stream_move_plan(
                    sink,
                    self.scan_id,
                    &layout,
                    &candidates.files,
                    &candidates.metadata,
                    folders,
                    reservations,
                );
                for plan in planned {
                    plans.insert(plan.file_id.clone(), plan);
                }
                self.moves_changed.notify_all();
            }
            Err(error) => {
                for (file_id, previous) in candidates.previous {
                    match previous {
                        Some(signature) => signatures.insert(file_id.clone(), signature),
                        None => signatures.remove(&file_id),
                    };
                    self.plan_error(sink, file_id, &error);
                }
            }
        }
    }

    /// Each file shows the error, and plans again once places are applied.
    pub(super) fn fail_all(&self, sink: &impl EventSink, file_ids: &[String], error: &str) {
        for file_id in file_ids {
            self.plan_error(sink, file_id.clone(), error);
        }
    }

    fn plan_error(&self, sink: &impl EventSink, file_id: String, error: &str) {
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

    fn candidates(&self, planning: &mut Planning, file_ids: &[String]) -> Candidates {
        let mut candidates = Candidates {
            files: Vec::new(),
            metadata: Vec::new(),
            previous: HashMap::new(),
        };
        let Some(folders) = planning.places.as_ref() else {
            return candidates;
        };
        for file in self.known_records(file_ids) {
            let media = matches!(file.kind, FileKind::Photo | FileKind::Video);
            let Some(metadata) = self.metadata_of(&file.id).filter(|_| media) else {
                continue;
            };
            let signature = plan_signature(&metadata, folders);
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

    fn layout(&self, with_places: bool) -> Result<DestinationLayout, String> {
        let context = &self.context;
        path_safety::validate_selected_folder(&context.folder)?;
        DestinationLayout::for_folder(&context.settings, &context.folder, with_places)
    }
}
