use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex, MutexGuard, PoisonError,
};

use tokio_util::sync::CancellationToken;

/// Cancellation handle of the most recently started scan.
#[derive(Default)]
pub(crate) struct ActiveScan {
    slot: Mutex<Option<ScanTicket>>,
    started_scans: AtomicU64,
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
        *self.lock() = Some(ticket.clone());
        ticket
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
