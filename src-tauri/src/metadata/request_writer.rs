use std::{io::Write, path::PathBuf};

use super::worker::WorkerError;

const EXIFTOOL_TAGS: [&str; 11] = [
    "-SubSecDateTimeOriginal",
    "-DateTimeOriginal",
    "-CreateDate",
    "-ModifyDate",
    "-MediaCreateDate",
    "-TrackCreateDate",
    "-FileModifyDate",
    "-Make",
    "-Model",
    "-DeviceModelName",
    "-CameraModelName",
];

/// Writes one `-stay_open` JSON request terminated by `-execute{request_id}`.
pub(super) fn write_request(
    stdin: &mut impl Write,
    source_paths: &[PathBuf],
    request_id: u64,
) -> Result<(), WorkerError> {
    stdin.write_all(b"-json\n").map_err(|error| {
        WorkerError::Crashed(format!("Failed to write ExifTool request: {error}"))
    })?;

    for tag in EXIFTOOL_TAGS {
        stdin
            .write_all(format!("{tag}\n").as_bytes())
            .map_err(|error| {
                WorkerError::Crashed(format!("Failed to write ExifTool tag request: {error}"))
            })?;
    }

    for source_path in source_paths {
        stdin
            .write_all(format!("{}\n", source_path.to_string_lossy()).as_bytes())
            .map_err(|error| {
                WorkerError::Crashed(format!("Failed to write ExifTool source path: {error}"))
            })?;
    }

    stdin
        .write_all(format!("-execute{request_id}\n").as_bytes())
        .map_err(|error| {
            WorkerError::Crashed(format!("Failed to write ExifTool execute marker: {error}"))
        })?;
    stdin
        .flush()
        .map_err(|error| WorkerError::Crashed(format!("Failed to flush ExifTool stdin: {error}")))
}
