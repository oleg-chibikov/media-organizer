use std::{
    path::PathBuf,
    sync::mpsc::RecvTimeoutError,
    time::{Duration, Instant},
};

use super::{
    exiftool_json::parse_exiftool_json_batch,
    output_reader::ReaderEvent,
    request_writer::write_request,
    result_alignment::align_results,
    worker::{ExifToolWorker, WorkerError},
    RawMetadataDates,
};

impl ExifToolWorker {
    /// Sends one request and waits for its framed response, one slot per source path.
    pub(super) fn read_metadata_batch(
        &mut self,
        source_paths: &[PathBuf],
        timeout: Duration,
    ) -> Result<Vec<Option<RawMetadataDates>>, WorkerError> {
        let request_id = self.next_request_id;
        self.next_request_id += 1;

        write_request(&mut self.stdin, source_paths, request_id)?;

        let timed_out = || {
            WorkerError::TimedOut(format!(
                "ExifTool request timed out after {}ms for {} file(s)",
                timeout.as_millis(),
                source_paths.len()
            ))
        };
        let deadline = Instant::now() + timeout;
        let now = Instant::now();
        if now >= deadline {
            return Err(timed_out());
        }

        match self
            .reader_rx
            .recv_timeout(deadline.saturating_duration_since(now))
        {
            Ok(ReaderEvent::Ready {
                request_id: ready_id,
                payload,
            }) => {
                if ready_id != request_id {
                    return Err(WorkerError::Protocol(format!(
                        "ExifTool response framing mismatch: expected {request_id}, got {ready_id}."
                    )));
                }
                let raw_dates =
                    parse_exiftool_json_batch(&payload).map_err(WorkerError::Protocol)?;
                Ok(align_results(source_paths, raw_dates))
            }
            Ok(ReaderEvent::Exited) => Err(WorkerError::Crashed(
                "ExifTool worker exited unexpectedly during metadata request.".to_string(),
            )),
            Err(RecvTimeoutError::Timeout) => Err(timed_out()),
            Err(RecvTimeoutError::Disconnected) => Err(WorkerError::Crashed(
                "ExifTool worker output channel disconnected unexpectedly.".to_string(),
            )),
        }
    }
}
