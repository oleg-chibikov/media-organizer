//! The only module allowed to rename, link, copy into or delete files.
//!
//! Every operation here refuses to replace an existing path, keeps the source until the
//! destination is verified, and deletes only files it created or proved to be redundant.
//! `clippy.toml` rejects the raw `std::fs` calls everywhere else.
#![allow(clippy::disallowed_methods)]

mod app_files;
mod content_check;
mod created_file;
mod move_file;
mod no_replace_rename;
mod occupancy;
mod removal;
mod staged_copy;

#[cfg(test)]
mod tests;

pub(crate) use app_files::{ensure_inside_dir, remove_app_file, replace_app_file};
pub(crate) use move_file::move_file_no_replace;
pub(crate) use occupancy::{is_plain_file, path_is_occupied};
pub(crate) use removal::{remove_empty_dir_within, remove_verified_duplicate};
pub(crate) use staged_copy::copy_verified_no_replace;
