//! Reads the capture date of every photo and video a scan finds, while the scan runs.

mod pause;
mod reader;
mod readers;
mod result_buffer;

#[cfg(test)]
mod tests;

use std::sync::{atomic::AtomicUsize, mpsc, Arc, Mutex};

use tokio_util::sync::CancellationToken;

use crate::{
    events::EventSink,
    models::{FileKind, FileRecord, MetadataReadRequest},
};
pub(crate) use pause::MetadataPause;
pub(crate) use reader::{BatchReader, QueueReader};
use readers::Readers;
use result_buffer::ResultBuffer;

/// Passes the scan's records to the readers. Dropping it tells them the scan has ended.
pub(crate) struct MetadataFeed {
    // Unbounded, so a slow read doesn't stall the scan. The rows are in memory anyway.
    requests: mpsc::Sender<MetadataReadRequest>,
}

impl MetadataFeed {
    pub(crate) fn push(&self, records: &[FileRecord]) {
        for record in records {
            if matches!(record.kind, FileKind::Photo | FileKind::Video) {
                // The readers only stop early on a cancelled scan, which stops the feed too.
                let _ = self.requests.send(MetadataReadRequest {
                    file_id: record.id.clone(),
                    source_path: record.source_path.clone(),
                    file_kind: record.kind.clone(),
                });
            }
        }
    }
}

pub(crate) struct ReadOptions<R> {
    pub scan_id: u64,
    pub token: CancellationToken,
    pub reader: Arc<R>,
    pub pause: Arc<MetadataPause>,
    pub reader_count: usize,
}

/// Starts the reader threads of one scan. They emit `metadata:results` as reads finish and
/// `metadata:finished` once the feed is dropped and drained, or the scan is cancelled.
pub(crate) fn start_reading<S, R>(sink: S, options: ReadOptions<R>) -> MetadataFeed
where
    S: EventSink + Send + Sync + 'static,
    R: BatchReader,
{
    let (sender, receiver) = mpsc::channel();
    let reader_count = options.reader_count.max(1);
    let readers = Arc::new(Readers {
        sink,
        scan_id: options.scan_id,
        token: options.token,
        reader: options.reader,
        pause: options.pause,
        requests: Mutex::new(receiver),
        buffer: ResultBuffer::new(options.scan_id),
        running: AtomicUsize::new(reader_count),
    });
    for _ in 0..reader_count {
        let readers = readers.clone();
        std::thread::spawn(move || readers.run());
    }
    MetadataFeed { requests: sender }
}
