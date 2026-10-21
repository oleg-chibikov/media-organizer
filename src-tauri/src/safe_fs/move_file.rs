use std::{fs, io, path::Path};

use same_file::is_same_file;

use super::no_replace_rename::rename_no_replace;

/// Moves a file without ever replacing `destination`.
///
/// Tries an exclusive rename first. Filesystems without one get a hard link, which also
/// fails on an existing destination, followed by removal of the old name. Errors keep their
/// kind: `AlreadyExists` for a taken destination, `CrossesDevices` across volumes and
/// `Unsupported` when neither strategy works here. In every error case the source stays put.
pub(crate) fn move_file_no_replace(source: &Path, destination: &Path) -> io::Result<()> {
    match rename_no_replace(source, destination) {
        Err(rename_error) if rename_error.kind() == io::ErrorKind::Unsupported => {
            move_by_hard_link(source, destination).map_err(|link_error| match link_error.kind() {
                io::ErrorKind::AlreadyExists | io::ErrorKind::CrossesDevices => link_error,
                _ => io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("{rename_error}; hard link fallback failed: {link_error}"),
                ),
            })
        }
        result => result,
    }
}

/// Links `destination` to the source file, then drops the source name.
/// If the source name can't be removed, the new link goes instead, so one name always remains.
pub(super) fn move_by_hard_link(source: &Path, destination: &Path) -> io::Result<()> {
    fs::hard_link(source, destination)?;
    if !is_same_file(source, destination)? {
        return Err(io::Error::other(
            "hard link points at a different file; both files were kept",
        ));
    }
    if let Err(error) = fs::remove_file(source) {
        if is_same_file(source, destination).unwrap_or(false) {
            let _ = fs::remove_file(destination);
        }
        return Err(error);
    }
    Ok(())
}
