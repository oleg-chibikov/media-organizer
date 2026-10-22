use std::time::{Duration, Instant};

use crate::{
    events::{self, EventSink},
    models::{FileRecord, ScanFilesEvent},
};

const MAX_BATCH_RECORDS: usize = 500;
const MAX_BATCH_AGE: Duration = Duration::from_millis(100);

/// Groups scanned records into one event each, so a folder of 50k files costs
/// a few hundred IPC messages instead of 50k.
pub(super) struct RecordBatch<F> {
    records: Vec<FileRecord>,
    started: Instant,
    on_flush: F,
}

impl<F: FnMut(&[FileRecord])> RecordBatch<F> {
    pub(super) fn new(on_flush: F) -> Self {
        Self {
            records: Vec::new(),
            started: Instant::now(),
            on_flush,
        }
    }

    pub(super) fn push(&mut self, sink: &impl EventSink, record: FileRecord) {
        if self.records.is_empty() {
            self.started = Instant::now();
        }
        self.records.push(record);
        if self.records.len() >= MAX_BATCH_RECORDS || self.started.elapsed() >= MAX_BATCH_AGE {
            self.flush(sink);
        }
    }

    pub(super) fn flush(&mut self, sink: &impl EventSink) {
        if !self.records.is_empty() {
            let records = std::mem::take(&mut self.records);
            events::scan_files(
                sink,
                ScanFilesEvent {
                    records: records.clone(),
                },
            );
            (self.on_flush)(&records);
        }
    }
}
