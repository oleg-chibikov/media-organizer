//! Original-name recovery from organized filenames, Windows filename sanitization,
//! folder name checks and the place part of a file name.

mod folder_name;
mod original_name;
mod paren_groups;
mod place_label;
mod windows_name;

#[cfg(test)]
mod tests;

pub(crate) use folder_name::validated_folder_name;
pub(crate) use original_name::{original_name_from_stem, parse_original_name};
pub(crate) use place_label::place_label;
pub(crate) use windows_name::sanitize_for_windows_filename;
