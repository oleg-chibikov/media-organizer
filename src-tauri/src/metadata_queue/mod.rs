//! Bounded metadata read queue over a pool of persistent ExifTool workers.

mod admission;
mod batch;
mod limits;
mod results;
mod worker_pick;

#[cfg(test)]
mod tests;

use std::{
    sync::{atomic::AtomicUsize, Arc, Mutex},
    time::Duration,
};

use tokio::sync::Semaphore;

use crate::metadata::ExifToolWorkerManager;
pub(crate) use results::metadata_error_result;
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
    pub(crate) fn concurrency(&self) -> usize {
        self.workers.len()
    }
}
