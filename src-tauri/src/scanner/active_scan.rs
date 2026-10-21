use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex, MutexGuard, PoisonError,
};

use tokio_util::sync::CancellationToken;

/// The most recently started scan. Starting a scan cancels the one before it, and scans
/// run one at a time, so a new folder's events can't mix with an old folder's.
#[derive(Default)]
pub(crate) struct ActiveScan {
    slot: Mutex<Option<ScanTicket>>,
    started_scans: AtomicU64,
    running: Mutex<()>,
}

/// Identifies one started scan so a finished older scan cannot clear a newer one.
#[derive(Clone)]
pub(crate) struct ScanTicket {
    generation: u64,
    pub token: CancellationToken,
}

impl ActiveScan {
    pub(crate) fn begin(&self) -> ScanTicket {
        let ticket = ScanTicket {
            generation: self.started_scans.fetch_add(1, Ordering::Relaxed),
            token: CancellationToken::new(),
        };
        if let Some(previous) = self.lock().replace(ticket.clone()) {
            previous.token.cancel();
        }
        ticket
    }

    /// Runs `scan` once the previous scan has finished, then clears the ticket.
    pub(crate) fn run(&self, ticket: &ScanTicket, scan: impl FnOnce(&CancellationToken)) {
        let _one_at_a_time = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        scan(&ticket.token);
        self.finish(ticket);
    }

    pub(crate) fn finish(&self, ticket: &ScanTicket) {
        let mut slot = self.lock();
        if slot
            .as_ref()
            .is_some_and(|current| current.generation == ticket.generation)
        {
            *slot = None;
        }
    }

    pub(crate) fn cancel(&self) {
        if let Some(ticket) = self.lock().as_ref() {
            ticket.token.cancel();
        }
    }

    fn lock(&self) -> MutexGuard<'_, Option<ScanTicket>> {
        self.slot.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
