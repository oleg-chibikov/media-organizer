use std::collections::HashMap;

use crate::models::{FileRecord, MetadataResult, MovePlan};

/// Per-file indexes over the request payload, keyed by file id.
pub(super) struct RunLookups<'a> {
    pub metadata_by_id: HashMap<&'a str, &'a MetadataResult>,
    pub plans_by_id: HashMap<&'a str, &'a MovePlan>,
    pub files_by_id: HashMap<&'a str, &'a FileRecord>,
}

impl<'a> RunLookups<'a> {
    pub(super) fn new(
        files: &'a [FileRecord],
        metadata_results: &'a [MetadataResult],
        plans: &'a [MovePlan],
    ) -> Self {
        Self {
            metadata_by_id: metadata_results
                .iter()
                .map(|metadata| (metadata.file_id.as_str(), metadata))
                .collect(),
            plans_by_id: plans
                .iter()
                .map(|plan| (plan.file_id.as_str(), plan))
                .collect(),
            files_by_id: files.iter().map(|file| (file.id.as_str(), file)).collect(),
        }
    }
}
