//! Centralized path checks used before reading from or writing to user folders.

use std::{
    fs,
    path::{Component, Path},
};

const PROCESSED_SEGMENT: &str = "Processed";

pub(crate) fn validate_selected_folder(selected_folder: &Path) -> Result<(), String> {
    if !selected_folder.exists() {
        return Err("Selected folder does not exist.".to_string());
    }
    if !selected_folder.is_dir() {
        return Err("Selected path is not a folder.".to_string());
    }
    if path_contains_processed_segment(selected_folder) {
        return Err("Selected folder must not be inside a Processed path.".to_string());
    }

    fs::read_dir(selected_folder)
        .map_err(|error| format!("Selected folder is not readable: {error}"))?;

    Ok(())
}

pub(crate) fn path_contains_processed_segment(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(PROCESSED_SEGMENT)
    })
}

pub(crate) fn has_parent_dir_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

pub(crate) fn path_is_within_case_insensitive(candidate: &Path, parent: &Path) -> bool {
    let candidate = normalized_path_string(candidate);
    let parent = normalized_path_string(parent);

    candidate == parent || candidate.starts_with(&format!("{parent}/"))
}

pub(crate) fn paths_equal_case_insensitive(left: &Path, right: &Path) -> bool {
    normalized_path_string(left) == normalized_path_string(right)
}

pub(crate) fn normalized_path_string(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase()
}
