use std::sync::atomic::{AtomicU64, Ordering};

use chrono::{DateTime, Utc};

static RUN_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static OPERATION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn generate_run_id(now: DateTime<Utc>) -> String {
    let sequence = RUN_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "{:x}{:x}{:x}",
        now.timestamp_micros().unsigned_abs(),
        std::process::id(),
        sequence
    )
}

/// Zero-padded hex keeps operation ids sortable in execution order.
pub(crate) fn next_operation_id(run_id: &str) -> String {
    let sequence = OPERATION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("{run_id}-{:016x}", sequence)
}
