use std::path::PathBuf;

use super::results::metadata_error_result;
use crate::{
    metadata::exiftool_path_argument,
    models::{MetadataReadRequest, MetadataResult},
};

/// A request whose source file still exists and must be read by a worker.
pub(super) struct PendingRead {
    pub index: usize,
    pub request: MetadataReadRequest,
    pub source: PathBuf,
}

pub(super) fn queue_full_results(
    requests: Vec<MetadataReadRequest>,
    capacity: usize,
) -> Vec<MetadataResult> {
    requests
        .into_iter()
        .map(|request| {
            metadata_error_result(
                request.file_id,
                format!("Metadata queue is full (capacity {capacity}). Try again shortly."),
            )
        })
        .collect()
}

pub(super) fn split_missing_sources(
    requests: Vec<MetadataReadRequest>,
) -> (Vec<Option<MetadataResult>>, Vec<PendingRead>) {
    let mut results: Vec<Option<MetadataResult>> = vec![None; requests.len()];
    let mut pending = Vec::new();

    for (index, request) in requests.into_iter().enumerate() {
        let source = PathBuf::from(&request.source_path);
        if let Err(reason) = exiftool_path_argument(&source) {
            results[index] = Some(metadata_error_result(request.file_id, reason));
        } else if source.exists() {
            pending.push(PendingRead {
                index,
                request,
                source,
            });
        } else {
            results[index] = Some(metadata_error_result(
                request.file_id,
                "File no longer exists on disk.".to_string(),
            ));
        }
    }

    (results, pending)
}
