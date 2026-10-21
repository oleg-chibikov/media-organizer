use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use super::{
    worker::{ExifToolWorker, WorkerError},
    MetadataBatchReadOutcome,
};

/// Owns one ExifTool worker, restarting it once after a crash.
#[derive(Default)]
pub struct ExifToolWorkerManager {
    worker: Option<ExifToolWorker>,
}

impl ExifToolWorkerManager {
    /// `resource_dir` is where the bundled ExifTool sidecar is looked for first.
    pub fn read_metadata_batch(
        &mut self,
        resource_dir: Option<&Path>,
        source_paths: &[PathBuf],
        timeout: Duration,
    ) -> Result<MetadataBatchReadOutcome, String> {
        if source_paths.is_empty() {
            return Ok(MetadataBatchReadOutcome {
                raw_dates: Vec::new(),
                did_restart: false,
            });
        }

        let mut did_restart = false;

        for attempt in 0..=1 {
            let had_worker = self.worker.is_some();
            if self.worker_needs_restart() {
                self.shutdown_worker();
                let worker = ExifToolWorker::start(resource_dir)
                    .inspect_err(|error| log::error!("{error}"))?;
                self.worker = Some(worker);
                if had_worker {
                    did_restart = true;
                }
            }

            let Some(worker) = self.worker.as_mut() else {
                return Err("ExifTool worker is unavailable.".to_string());
            };

            match worker.read_metadata_batch(source_paths, timeout) {
                Ok(raw_dates) => {
                    return Ok(MetadataBatchReadOutcome {
                        raw_dates,
                        did_restart,
                    });
                }
                Err(WorkerError::Crashed(message)) => {
                    log::warn!("ExifTool crashed: {message}");
                    self.shutdown_worker();
                    if attempt == 0 {
                        did_restart = true;
                        continue;
                    }
                    return Err(message);
                }
                Err(WorkerError::TimedOut(message) | WorkerError::Protocol(message)) => {
                    log::warn!("ExifTool failed: {message}");
                    self.shutdown_worker();
                    return Err(message);
                }
            }
        }

        Err("ExifTool worker failed after restart attempt.".to_string())
    }

    fn worker_needs_restart(&mut self) -> bool {
        match self.worker.as_mut() {
            Some(worker) => !worker.is_alive(),
            None => true,
        }
    }

    fn shutdown_worker(&mut self) {
        if let Some(mut worker) = self.worker.take() {
            worker.shutdown();
        }
    }
}
