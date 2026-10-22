use std::{
    sync::{Mutex, PoisonError},
    time::{Duration, Instant},
};

use crate::{
    events::{self, EventSink},
    models::{MetadataResult, MetadataResultsEvent},
};

const MAX_RESULTS: usize = 500;
const MAX_AGE: Duration = Duration::from_millis(250);

/// Groups the results of every reader into one event, so a large folder costs a few events
/// a second instead of one per batch.
pub(super) struct ResultBuffer {
    scan_id: u64,
    pending: Mutex<Pending>,
}

struct Pending {
    results: Vec<MetadataResult>,
    since: Instant,
}

impl ResultBuffer {
    pub(super) fn new(scan_id: u64) -> Self {
        let pending = Pending {
            results: Vec::new(),
            since: Instant::now(),
        };
        Self {
            scan_id,
            pending: Mutex::new(pending),
        }
    }

    pub(super) fn push(&self, sink: &impl EventSink, results: Vec<MetadataResult>) {
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        if pending.results.is_empty() {
            pending.since = Instant::now();
        }
        pending.results.extend(results);
        if pending.results.len() >= MAX_RESULTS || pending.since.elapsed() >= MAX_AGE {
            self.emit(sink, std::mem::take(&mut pending.results));
        }
    }

    pub(super) fn flush(&self, sink: &impl EventSink) {
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        self.emit(sink, std::mem::take(&mut pending.results));
    }

    fn emit(&self, sink: &impl EventSink, results: Vec<MetadataResult>) {
        if !results.is_empty() {
            let scan_id = self.scan_id;
            events::metadata_results(sink, MetadataResultsEvent { scan_id, results });
        }
    }
}
