use std::sync::{Arc, Mutex, MutexGuard, PoisonError, TryLockError};

use crate::metadata::ExifToolWorkerManager;

pub(super) type SharedWorker = Arc<Mutex<ExifToolWorkerManager>>;

/// Locks the first idle worker from `start` on, so a batch doesn't queue behind a busy
/// worker while another sits idle. Waits for the worker at `start` when all are busy.
/// A worker poisoned by a panic is still usable: the manager restarts a dead process.
pub(super) fn lock_idle_worker(
    workers: &[SharedWorker],
    start: usize,
) -> MutexGuard<'_, ExifToolWorkerManager> {
    let count = workers.len();
    (0..count)
        .map(|offset| &workers[(start + offset) % count])
        .find_map(|worker| match worker.try_lock() {
            Ok(guard) => Some(guard),
            Err(TryLockError::Poisoned(poisoned)) => Some(poisoned.into_inner()),
            Err(TryLockError::WouldBlock) => None,
        })
        .unwrap_or_else(|| {
            workers[start % count]
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
        })
}
