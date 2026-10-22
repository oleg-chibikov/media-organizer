use super::Planning;
use crate::{
    events::{self, EventSink},
    models::PlacedFilesEvent,
    planner,
    scan_session::ScanSession,
};

impl ScanSession {
    /// Swaps the placed files the last place folders moved for the ones the new folders
    /// continue. Their records and dates join the scan, and their plans come with them.
    pub(super) fn plan_placed(
        &self,
        sink: &impl EventSink,
        planning: &mut Planning,
    ) -> Result<(), String> {
        let dropped: Vec<String> = planning.placed_ids.drain().collect();
        for file_id in &dropped {
            planning.plans.remove(file_id);
            planning.signatures.remove(file_id);
        }
        self.forget(&dropped);
        let places = planning.places.as_deref().unwrap_or_default();
        let continues = places
            .iter()
            .any(|suggestion| suggestion.continues.is_some());
        let context = &self.context;
        let added = if continues {
            planner::plan_continued_moves(
                &context.settings,
                &context.placements,
                &context.folder,
                places,
                &self.all_metadata(),
            )
        } else {
            Ok(Vec::new())
        };
        let (added, result) = match added {
            Ok(added) => (added, Ok(())),
            Err(error) => (
                Vec::new(),
                Err(format!("Failed to plan placed files: {error}")),
            ),
        };
        for placed in &added {
            self.add_records(std::slice::from_ref(&placed.record));
            self.add_metadata(std::slice::from_ref(&placed.metadata));
            planning.placed_ids.insert(placed.record.id.clone());
            planning
                .plans
                .insert(placed.record.id.clone(), placed.plan.clone());
        }
        if !added.is_empty() || !dropped.is_empty() {
            let scan_id = self.scan_id;
            events::plan_placed(
                sink,
                PlacedFilesEvent {
                    scan_id,
                    added,
                    dropped,
                },
            );
        }
        result
    }
}
