use std::{path::PathBuf, time::Duration};

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
    pub fn read_metadata_batch(
        &mut self,
        app: &tauri::AppHandle,
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
                self.worker = Some(ExifToolWorker::start(app)?);
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
                    self.shutdown_worker();
                    if attempt == 0 {
                        did_restart = true;
                        continue;
                    }
                    return Err(message);
                }
                Err(WorkerError::TimedOut(message) | WorkerError::Protocol(message)) => {
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
