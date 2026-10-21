use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::path_safety::{
    normalized_path_string, path_is_within_case_insensitive, paths_equal_case_insensitive,
};

/// Removes source folders emptied by moves, never climbing above the selected root.
pub(super) fn cleanup_empty_source_directories(
    selected_root: &Path,
    source_directories: &[PathBuf],
) {
    let mut visited = HashSet::new();
    let mut cleaned_any_directory = false;

    for directory in source_directories {
        let key = normalized_path_string(directory);
        if visited.insert(key) {
            prune_empty_directory_chain(selected_root, directory);
            cleaned_any_directory = true;
        }
    }

    if cleaned_any_directory {
        remove_empty_selected_root(selected_root);
    }
}

fn prune_empty_directory_chain(selected_root: &Path, start_directory: &Path) {
    let mut current = start_directory.to_path_buf();

    loop {
        if paths_equal_case_insensitive(&current, selected_root)
            || !path_is_within_case_insensitive(&current, selected_root)
        {
            break;
        }

        if !current.is_dir() || !is_empty_directory(&current) {
            break;
        }

        if fs::remove_dir(&current).is_err() {
            break;
        }

        let Some(parent) = current.parent() else {
            break;
        };
        current = parent.to_path_buf();
    }
}

fn remove_empty_selected_root(selected_root: &Path) {
    if selected_root.is_dir() && is_empty_directory(selected_root) {
        let _ = fs::remove_dir(selected_root);
    }
}

fn is_empty_directory(directory: &Path) -> bool {
    match fs::read_dir(directory) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => false,
    }
}
