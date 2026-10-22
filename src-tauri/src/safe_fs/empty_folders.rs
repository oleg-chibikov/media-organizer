use std::{
    fs,
    path::{Path, PathBuf},
};

use same_file::Handle;

use super::occupancy::is_plain_directory;

/// Removes every empty real folder under `root`, deepest first, then `root` once it is empty.
/// Only `remove_dir` runs, and it refuses a folder that still holds anything, so files stay.
/// Links are not followed, and `keep` is skipped together with everything inside it.
/// Returns whether `root` itself was removed.
pub(crate) fn remove_empty_folders(root: &Path, keep: &Path) -> bool {
    remove_empty_tree(root, Handle::from_path(keep).ok().as_ref())
}

/// Removes `start` and then each parent while it is empty, stopping below `stop`, which
/// stays. Only folders strictly inside `stop` are touched, and only with `remove_dir`.
pub(crate) fn remove_empty_parents(start: &Path, stop: &Path) {
    let mut directory = start;
    while directory != stop && directory.starts_with(stop) {
        if !is_plain_directory(directory) || fs::remove_dir(directory).is_err() {
            return;
        }
        let Some(parent) = directory.parent() else {
            return;
        };
        directory = parent;
    }
}

fn remove_empty_tree(directory: &Path, keep: Option<&Handle>) -> bool {
    if keep.is_some_and(|keep| Handle::from_path(directory).is_ok_and(|handle| handle == *keep)) {
        return false;
    }
    for subfolder in plain_subfolders(directory) {
        remove_empty_tree(&subfolder, keep);
    }
    is_plain_directory(directory) && fs::remove_dir(directory).is_ok()
}

// Collected up front so no folder handle stays open while its children are removed.
fn plain_subfolders(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| is_plain_directory(path))
        .collect()
}
