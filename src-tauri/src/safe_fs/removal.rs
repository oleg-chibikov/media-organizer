use std::{fs, io, path::Path};

use same_file::is_same_file;

use super::{content_check::verify_same_content, occupancy::is_plain_file};

/// Deletes `duplicate` only when `original` is a different regular file with the same bytes,
/// so the content survives. Links and second names of one file are refused.
pub(crate) fn remove_verified_duplicate(duplicate: &Path, original: &Path) -> io::Result<()> {
    if !is_plain_file(duplicate) || !is_plain_file(original) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "both paths must be regular files, not links",
        ));
    }
    if is_same_file(duplicate, original)? {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "both paths lead to the same file",
        ));
    }
    verify_same_content(original, duplicate)?;
    fs::remove_file(duplicate)
}
