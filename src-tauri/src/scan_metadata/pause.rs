use std::{
    sync::{Condvar, Mutex, PoisonError},
    time::Duration,
};

use tokio_util::sync::CancellationToken;

// A cancelled scan wakes a paused reader within this long.
const CANCEL_CHECK: Duration = Duration::from_millis(200);

/// Holds metadata reads while a move run is paused, so the computer gets its CPU back.
/// Reads already running finish.
#[derive(Default)]
pub(crate) struct MetadataPause {
    paused: Mutex<bool>,
    resumed: Condvar,
}

impl MetadataPause {
    pub(crate) fn set(&self, paused: bool) {
        *self.paused.lock().unwrap_or_else(PoisonError::into_inner) = paused;
        if !paused {
            self.resumed.notify_all();
        }
    }

    /// Waits for a resume. Returns false when the scan was cancelled first.
    pub(super) fn wait_until_resumed(&self, token: &CancellationToken) -> bool {
        let mut paused = self.paused.lock().unwrap_or_else(PoisonError::into_inner);
        while *paused {
            if token.is_cancelled() {
                return false;
            }
            paused = self
                .resumed
                .wait_timeout(paused, CANCEL_CHECK)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
        !token.is_cancelled()
    }
}
