use std::{path::PathBuf, sync::Arc};

use crate::{
    metadata_queue::{metadata_error_result, MetadataQueue},
    models::{MetadataReadRequest, MetadataResult},
};

/// Reads one batch and answers every request, with an error result where the read failed.
pub(crate) trait BatchReader: Send + Sync + 'static {
    fn read(&self, requests: Vec<MetadataReadRequest>) -> Vec<MetadataResult>;

    /// Called once every reader of the scan has stopped.
    fn finished(&self) {}
}

/// Reads through the shared ExifTool workers.
pub(crate) struct QueueReader {
    pub queue: Arc<MetadataQueue>,
    pub resource_dir: Option<PathBuf>,
}

impl BatchReader for QueueReader {
    fn read(&self, requests: Vec<MetadataReadRequest>) -> Vec<MetadataResult> {
        let file_ids: Vec<String> = requests.iter().map(|r| r.file_id.clone()).collect();
        let read = self.queue.read_batch(self.resource_dir.clone(), requests);
        match tauri::async_runtime::block_on(read) {
            Ok(results) => results,
            Err(error) => file_ids
                .into_iter()
                .map(|file_id| metadata_error_result(file_id, error.clone()))
                .collect(),
        }
    }
}
