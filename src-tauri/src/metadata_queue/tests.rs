use std::sync::{Arc, Mutex, TryLockError};

use super::worker_pick::{lock_idle_worker, SharedWorker};

#[test]
fn picks_an_idle_worker_over_a_busy_one() {
    let workers: Vec<SharedWorker> = (0..2).map(|_| Arc::new(Mutex::default())).collect();
    let _busy = workers[0].lock().expect("lock first worker");

    let _picked = lock_idle_worker(&workers, 0);

    assert!(matches!(
        workers[1].try_lock(),
        Err(TryLockError::WouldBlock)
    ));
}
