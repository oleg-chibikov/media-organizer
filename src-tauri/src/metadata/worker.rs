use std::{
    io::Write,
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::Receiver,
};

use super::{
    exiftool_path::resolve_exiftool_path,
    output_reader::{spawn_stderr_drain, spawn_stdout_reader, ReaderEvent},
};

#[derive(Debug)]
pub(super) enum WorkerError {
    TimedOut(String),
    Crashed(String),
    Protocol(String),
}

/// A long-lived `exiftool -stay_open` process fed requests over stdin.
pub(super) struct ExifToolWorker {
    pub(super) child: Child,
    pub(super) stdin: ChildStdin,
    pub(super) reader_rx: Receiver<ReaderEvent>,
    pub(super) next_request_id: u64,
}

impl ExifToolWorker {
    pub(super) fn start(resource_dir: Option<&Path>) -> Result<Self, String> {
        let exiftool_path = resolve_exiftool_path(resource_dir)?;

        let mut child = Command::new(&exiftool_path)
            .args(["-stay_open", "True", "-@", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("Failed to start ExifTool at {:?}: {error}", exiftool_path))?;

        log::info!("Started ExifTool at {}", exiftool_path.display());

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to open ExifTool stdin.".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to open ExifTool stdout.".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Failed to open ExifTool stderr.".to_string())?;

        let reader_rx = spawn_stdout_reader(stdout);
        spawn_stderr_drain(stderr);

        Ok(Self {
            child,
            stdin,
            reader_rx,
            next_request_id: 1,
        })
    }

    pub(super) fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    pub(super) fn shutdown(&mut self) {
        let _ = self.stdin.write_all(b"-stay_open\nFalse\n");
        let _ = self.stdin.write_all(b"-execute\n");
        let _ = self.stdin.flush();

        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

impl Drop for ExifToolWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}
