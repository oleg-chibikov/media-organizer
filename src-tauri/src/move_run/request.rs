use std::{collections::HashMap, path::PathBuf};

use crate::{
    models::{FileRecord, MetadataResult, MovePlan},
    placements::PlacementStore,
};

/// One confirmed batch as the frontend sends it.
pub(crate) struct MoveBatch {
    pub files: Vec<FileRecord>,
    pub metadata_results: Vec<MetadataResult>,
    pub plans: Vec<MovePlan>,
}

/// A confirmed batch with the roots it was validated against.
pub(super) struct MoveRunRequest {
    pub selected_folder: PathBuf,
    pub destination_root: PathBuf,
    /// Where place folders go. The destination root unless renaming in place.
    pub place_root: PathBuf,
    pub batch: MoveBatch,
    /// Where the files moved into place folders get recorded.
    pub placements: PlacementStore,
}

/// Per-file indexes over the request payload, keyed by file id.
pub(super) struct RunLookups<'a> {
    pub metadata_by_id: HashMap<&'a str, &'a MetadataResult>,
    pub plans_by_id: HashMap<&'a str, &'a MovePlan>,
    pub files_by_id: HashMap<&'a str, &'a FileRecord>,
}

impl<'a> RunLookups<'a> {
    pub(super) fn new(batch: &'a MoveBatch) -> Self {
        Self {
            metadata_by_id: batch
                .metadata_results
                .iter()
                .map(|metadata| (metadata.file_id.as_str(), metadata))
                .collect(),
            plans_by_id: batch
                .plans
                .iter()
                .map(|plan| (plan.file_id.as_str(), plan))
                .collect(),
            files_by_id: batch
                .files
                .iter()
                .map(|file| (file.id.as_str(), file))
                .collect(),
        }
    }
}
