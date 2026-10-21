use std::{io::Write, path::PathBuf};

use super::{argfile_path::exiftool_path_argument, worker::WorkerError};

/// Paths go out as UTF-8. Without this, ExifTool on Windows reads them in the ANSI code page
/// and can't find a file with Cyrillic or accented letters in its path.
pub(super) const READ_OPTIONS: [&str; 3] = ["-charset", "filename=utf8", "-json"];

pub(super) const EXIFTOOL_TAGS: [&str; 11] = [
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

/// Writes one read-only `-stay_open` JSON request terminated by `-execute{request_id}`.
/// The whole request is refused before anything is written if one path is unsafe.
pub(super) fn write_request(
    stdin: &mut impl Write,
    source_paths: &[PathBuf],
    request_id: u64,
) -> Result<(), WorkerError> {
    let path_arguments = source_paths
        .iter()
        .map(|path| exiftool_path_argument(path))
        .collect::<Result<Vec<_>, _>>()
        .map_err(WorkerError::Protocol)?;

    for argument in READ_OPTIONS.iter().chain(&EXIFTOOL_TAGS) {
        stdin
            .write_all(format!("{argument}\n").as_bytes())
            .map_err(|error| {
                WorkerError::Crashed(format!("Failed to write ExifTool request: {error}"))
            })?;
    }

    for path_argument in path_arguments {
        stdin
            .write_all(format!("{path_argument}\n").as_bytes())
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
