//! Bounded metadata read queue over a pool of persistent ExifTool workers.

mod admission;
mod batch;
mod limits;
mod results;

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use tokio::sync::Semaphore;

use crate::{metadata::ExifToolWorkerManager, models::ProcessingConfig};

pub(crate) use results::metadata_error_result;

pub(crate) struct MetadataQueue {
    workers: Vec<Arc<Mutex<ExifToolWorkerManager>>>,
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

    fn select_worker(&self) -> Arc<Mutex<ExifToolWorkerManager>> {
        let index = self.next_worker.fetch_add(1, Ordering::Relaxed);
        let slot = index % self.workers.len();
        self.workers[slot].clone()
    }
}
