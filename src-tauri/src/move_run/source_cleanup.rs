use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::{
    path_safety::{
        normalized_path_string, path_is_within_case_insensitive, paths_equal_case_insensitive,
    },
    safe_fs,
};

/// Removes source folders emptied by moves, walking up to just below the selected root.
/// The selected root itself is always kept.
pub(super) fn cleanup_empty_source_directories(
    selected_root: &Path,
    source_directories: &[PathBuf],
) {
    let mut visited = HashSet::new();
    for directory in source_directories {
        if visited.insert(normalized_path_string(directory)) {
            prune_empty_directory_chain(selected_root, directory);
        }
    }
}

fn prune_empty_directory_chain(selected_root: &Path, start_directory: &Path) {
    let mut current = start_directory;
    while !paths_equal_case_insensitive(current, selected_root)
        && path_is_within_case_insensitive(current, selected_root)
        && safe_fs::remove_empty_dir_within(current, selected_root)
    {
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }
}
