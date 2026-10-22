use std::collections::HashMap;

use super::{signature::plan_signature, Planning};
use crate::{
    events::{self, EventSink},
    models::{FileKind, FileRecord, MetadataResult, PlanFileError, PlanFilesEvent},
    path_safety,
    planner::{self, DestinationLayout, PlannedFiles},
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
        let layout = match self.layout(planning) {
            Ok(layout) => layout,
            Err(error) => {
                for (file_id, previous) in &candidates.previous {
                    match previous {
                        Some(signature) => planning
                            .signatures
                            .insert(file_id.clone(), signature.clone()),
                        None => planning.signatures.remove(file_id),
                    };
                }
                let file_ids: Vec<String> = candidates.previous.into_keys().collect();
                return self.fail_all(sink, &file_ids, &error);
            }
        };
        let Planning {
            places,
            plans,
            reservations,
            ..
        } = planning;
        let Some(folders) = places.as_ref() else {
            return;
        };
        let planned = planner::plan_files(
            &layout,
            &candidates.files,
            &candidates.metadata,
            folders,
            reservations,
        );
        for plan in &planned.plans {
            plans.insert(plan.file_id.clone(), plan.clone());
        }
        self.emit_planned(sink, planned);
        self.moves_changed.notify_all();
    }

    /// Each file shows the error, and plans again once places are applied.
    pub(super) fn fail_all(&self, sink: &impl EventSink, file_ids: &[String], error: &str) {
        let errors = file_ids
            .iter()
            .map(|file_id| PlanFileError {
                file_id: file_id.clone(),
                error: error.to_string(),
            })
            .collect();
        self.emit_planned(
            sink,
            PlannedFiles {
                plans: Vec::new(),
                errors,
            },
        );
    }

    fn emit_planned(&self, sink: &impl EventSink, planned: PlannedFiles) {
        let PlannedFiles { plans, errors } = planned;
        let scan_id = self.scan_id;
        events::plan_files(
            sink,
            PlanFilesEvent {
                scan_id,
                plans,
                errors,
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

    /// The layout is resolved once per set of places, since it reads the settings and the folder.
    fn layout(&self, planning: &mut Planning) -> Result<DestinationLayout, String> {
        if let Some(layout) = &planning.layout {
            return Ok(layout.clone());
        }
        let context = &self.context;
        let with_places = planning
            .places
            .as_ref()
            .is_some_and(|folders| !folders.is_empty());
        path_safety::validate_selected_folder(&context.folder)?;
        let layout =
            DestinationLayout::for_folder(&context.settings, &context.folder, with_places)?;
        planning.layout = Some(layout.clone());
        Ok(layout)
    }
}
