use std::{collections::HashMap, path::PathBuf};

use super::request::MoveRunRequest;
use crate::{
    models::{MoveResult, MoveResultStatus},
    placements::{record_moves, MovedIntoPlace},
};

/// Records the files that reached a place folder. Skipped and failed files stay out.
/// A failed save only means the next scan starts a new trip folder.
pub(super) fn record_placements(request: &MoveRunRequest, results: &[MoveResult]) {
    let batch = &request.batch;
    let plans: HashMap<&str, _> = batch
        .plans
        .iter()
        .map(|plan| (plan.file_id.as_str(), plan))
        .collect();
    let dates: HashMap<&str, _> = batch
        .metadata_results
        .iter()
        .filter_map(|metadata| Some((metadata.file_id.as_str(), metadata.chosen_date.as_ref()?)))
        .collect();
    let moves: Vec<MovedIntoPlace> = results
        .iter()
        .filter(|result| {
            matches!(
                result.result_status,
                MoveResultStatus::Moved | MoveResultStatus::Copied
            )
        })
        .filter_map(|result| {
            let file_id = result.file_id.as_str();
            Some(MovedIntoPlace {
                placement: plans.get(file_id)?.placement.clone()?,
                source: PathBuf::from(&result.attempted_source_path),
                destination: PathBuf::from(&result.attempted_destination_path),
                chosen_date: (*dates.get(file_id)?).clone(),
            })
        })
        .collect();
    if moves.is_empty() {
        return;
    }
    let root = &request.destination_root;
    if let Err(error) = request
        .placements
        .update(|records| record_moves(records, root, &moves))
    {
        log::warn!("Failed to record placed files: {error}");
    }
}
