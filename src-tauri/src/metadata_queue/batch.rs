use std::{path::PathBuf, sync::atomic::Ordering};

use super::{
    admission::{queue_full_results, split_missing_sources, PendingRead},
    results::{
        metadata_batch_timeout, metadata_error_result, metadata_result_from_raw,
        unwrap_metadata_results,
    },
    MetadataQueue,
};
use crate::{
    metadata::MetadataBatchReadOutcome,
    models::{MetadataReadRequest, MetadataResult},
};

impl MetadataQueue {
    /// Reads one batch on a single worker, returning one result per request in request order.
    pub(crate) async fn read_batch(
        &self,
        resource_dir: Option<PathBuf>,
        requests: Vec<MetadataReadRequest>,
    ) -> Result<Vec<MetadataResult>, String> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        let request_count = requests.len();
        let queued = self.waiting.fetch_add(request_count, Ordering::SeqCst) + request_count;
        if queued > self.capacity {
            self.waiting.fetch_sub(request_count, Ordering::SeqCst);
            return Ok(queue_full_results(requests, self.capacity));
        }

        let (mut results, pending) = split_missing_sources(requests);
        if pending.is_empty() {
            self.waiting.fetch_sub(request_count, Ordering::SeqCst);
            return Ok(unwrap_metadata_results(results));
        }

        let permit = self
            .slots
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| "Metadata queue is unavailable.".to_string())?;
        self.waiting.fetch_sub(request_count, Ordering::SeqCst);

        let read_result = self.read_on_worker(resource_dir, &pending).await?;
        drop(permit);

        match read_result {
            Ok(outcome) => {
                for (pending, raw_dates) in pending.into_iter().zip(outcome.raw_dates) {
                    results[pending.index] = Some(metadata_result_from_raw(
                        pending.request,
                        raw_dates,
                        outcome.did_restart,
                    ));
                }
            }
            Err(error) => {
                for pending in pending {
                    results[pending.index] = Some(metadata_error_result(
                        pending.request.file_id,
                        error.clone(),
                    ));
                }
            }
        }

        Ok(unwrap_metadata_results(results))
    }

    async fn read_on_worker(
        &self,
        resource_dir: Option<PathBuf>,
        pending: &[PendingRead],
    ) -> Result<Result<MetadataBatchReadOutcome, String>, String> {
        let worker = self.select_worker();
        let timeout = metadata_batch_timeout(self.timeout, pending.len());
        let source_paths: Vec<PathBuf> = pending.iter().map(|read| read.source.clone()).collect();

        tauri::async_runtime::spawn_blocking(move || {
            let mut manager = worker
                .lock()
                .map_err(|_| "Failed to lock ExifTool worker state.".to_string())?;
            manager.read_metadata_batch(resource_dir.as_deref(), &source_paths, timeout)
        })
        .await
        .map_err(|error| format!("Metadata worker task failed: {error}"))
    }
}
