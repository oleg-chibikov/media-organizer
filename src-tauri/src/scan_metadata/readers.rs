use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc::Receiver,
    Arc, Mutex, PoisonError,
};

use tokio_util::sync::CancellationToken;

use super::{result_buffer::ResultBuffer, BatchReader, MetadataPause};
use crate::{
    events::{self, EventSink},
    models::{MetadataFinishedEvent, MetadataReadRequest},
};

// Small batches keep the table filling in steadily. ExifTool takes as long per file either way.
const BATCH_SIZE: usize = 8;

/// What the readers of one scan share. The last reader to stop reports the scan finished.
pub(super) struct Readers<S, R> {
    pub sink: S,
    pub scan_id: u64,
    pub token: CancellationToken,
    pub reader: Arc<R>,
    pub pause: Arc<MetadataPause>,
    pub requests: Mutex<Receiver<MetadataReadRequest>>,
    pub buffer: ResultBuffer,
    pub running: AtomicUsize,
}

impl<S: EventSink, R: BatchReader> Readers<S, R> {
    pub(super) fn run(&self) {
        while self.pause.wait_until_resumed(&self.token) {
            let batch = self.take_batch();
            if batch.is_empty() || self.token.is_cancelled() {
                break;
            }
            let results = self.reader.read(batch);
            // A replaced scan's dates belong to files no longer on screen.
            if self.token.is_cancelled() {
                break;
            }
            self.buffer.push(&self.sink, results);
        }
        if self.running.fetch_sub(1, Ordering::SeqCst) == 1 {
            self.finish();
        }
    }

    /// Waits for the next request, then takes the ones already queued behind it.
    /// Empty once the scan has ended and every request is taken.
    fn take_batch(&self) -> Vec<MetadataReadRequest> {
        let requests = self.requests.lock().unwrap_or_else(PoisonError::into_inner);
        let Ok(first) = requests.recv() else {
            return Vec::new();
        };
        let mut batch = vec![first];
        while batch.len() < BATCH_SIZE {
            match requests.try_recv() {
                Ok(request) => batch.push(request),
                Err(_) => break,
            }
        }
        batch
    }

    fn finish(&self) {
        self.reader.finished();
        let cancelled = self.token.is_cancelled();
        if !cancelled {
            self.buffer.flush(&self.sink);
        }
        let scan_id = self.scan_id;
        events::metadata_finished(&self.sink, MetadataFinishedEvent { scan_id, cancelled });
    }
}
