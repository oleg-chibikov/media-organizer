use crate::{models::MetadataResult, planner::PlaceFolders};

/// A file is planned again only when its date or its place folder changes its destination.
pub(super) fn plan_signature(metadata: &MetadataResult, folders: &PlaceFolders) -> String {
    format!(
        "{:?}:{}:{}",
        metadata.metadata_status,
        metadata.chosen_date.as_deref().unwrap_or(""),
        folders.key(&metadata.file_id)
    )
}
