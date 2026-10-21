use std::{fs, io, path::Path};

/// True when anything sits at `path`, including a dangling symlink.
/// Unreadable paths count as occupied so callers never treat them as free.
pub(crate) fn path_is_occupied(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Ok(_) => true,
        Err(error) => error.kind() != io::ErrorKind::NotFound,
    }
}

/// True for a regular file reached without following a symlink.
pub(crate) fn is_plain_file(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_file())
}
