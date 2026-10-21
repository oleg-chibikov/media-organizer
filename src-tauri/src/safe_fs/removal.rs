use std::{fs, io, path::Path};

use same_file::is_same_file;

use super::{
    content_check::verify_same_content,
    occupancy::{is_plain_directory, is_plain_file},
};

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

/// Removes `directory` when it is a real, empty folder strictly inside `root`.
/// `remove_dir` refuses non-empty folders, so a file arriving after the checks stops it.
/// Returns whether the folder was removed.
pub(crate) fn remove_empty_dir_within(directory: &Path, root: &Path) -> bool {
    if !is_plain_directory(directory) {
        return false;
    }
    let (Ok(real_directory), Ok(real_root)) = (fs::canonicalize(directory), fs::canonicalize(root))
    else {
        return false;
    };
    if real_directory == real_root || !real_directory.starts_with(&real_root) {
        return false;
    }
    fs::remove_dir(directory).is_ok()
}
