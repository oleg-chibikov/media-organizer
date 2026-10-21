//! Bounded metadata read queue over a pool of persistent ExifTool workers.

mod admission;
mod batch;
mod limits;
mod results;
mod worker_pick;

#[cfg(test)]
mod tests;

use std::{
    path::PathBuf,
    sync::{atomic::AtomicUsize, Arc, Mutex},
    time::Duration,
};

use tokio::sync::Semaphore;

use crate::{
    metadata::ExifToolWorkerManager,
    models::{MetadataReadRequest, MetadataResult, ProcessingConfig},
};
use results::metadata_error_result;
use worker_pick::SharedWorker;

pub(crate) struct MetadataQueue {
    workers: Vec<SharedWorker>,
    next_worker: AtomicUsize,
    slots: Arc<Semaphore>,
    waiting: AtomicUsize,
    capacity: usize,
    timeout: Duration,
}

impl Default for MetadataQueue {
    fn default() -> Self {
        let limits = limits::QueueLimits::from_env();
        let workers = (0..limits.concurrency)
            .map(|_| Arc::new(Mutex::new(ExifToolWorkerManager::default())))
            .collect();

        Self {
            workers,
            next_worker: AtomicUsize::new(0),
            slots: Arc::new(Semaphore::new(limits.concurrency)),
            waiting: AtomicUsize::new(0),
            capacity: limits.queue_capacity,
            timeout: limits.timeout,
        }
    }
}

impl MetadataQueue {
    pub(crate) fn processing_config(&self) -> ProcessingConfig {
        ProcessingConfig {
            metadata_concurrency: self.workers.len(),
            metadata_queue_capacity: self.capacity,
            metadata_timeout_ms: self.timeout.as_millis() as u64,
        }
    }

    pub(crate) async fn read_one(
        &self,
        resource_dir: Option<PathBuf>,
        request: MetadataReadRequest,
    ) -> Result<MetadataResult, String> {
        let file_id = request.file_id.clone();
        let mut results = self.read_batch(resource_dir, vec![request]).await?;
        Ok(results.pop().unwrap_or_else(|| {
            metadata_error_result(file_id, "Metadata worker returned no result.".to_string())
        }))
    }
}
