use std::sync::{Mutex, MutexGuard, PoisonError};

use tokio_util::sync::CancellationToken;

/// The stop signal of the latest move run. A paused run ends after the file moving now,
/// and the files it didn't reach stay where they are.
#[derive(Default)]
pub(crate) struct MovePause {
    current: Mutex<CancellationToken>,
}

impl MovePause {
    /// Gives a new run its own signal, so a pause before it started doesn't stop it.
    pub(crate) fn begin(&self) -> CancellationToken {
        let token = CancellationToken::new();
        *self.lock() = token.clone();
        token
    }

    pub(crate) fn pause(&self) {
        self.lock().cancel();
    }

    fn lock(&self) -> MutexGuard<'_, CancellationToken> {
        self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
