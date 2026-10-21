//! Marks the runs a move, replay or delete is working on, so a second job can't touch the
//! same run log or move the same files at once.

use std::{
    collections::BTreeSet,
    sync::{Mutex, MutexGuard, PoisonError},
};

static BUSY_RUNS: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

/// Held for as long as a job works on one run; dropping it frees the run.
#[derive(Debug)]
pub(crate) struct RunLease {
    run_id: String,
}

impl RunLease {
    pub(crate) fn acquire(run_id: &str) -> Result<Self, String> {
        if busy_runs().insert(run_id.to_string()) {
            Ok(Self {
                run_id: run_id.to_string(),
            })
        } else {
            Err(format!(
                "History run '{run_id}' is still being processed. Try again when it finishes."
            ))
        }
    }
}

impl Drop for RunLease {
    fn drop(&mut self) {
        busy_runs().remove(&self.run_id);
    }
}

fn busy_runs() -> MutexGuard<'static, BTreeSet<String>> {
    BUSY_RUNS.lock().unwrap_or_else(PoisonError::into_inner)
}
