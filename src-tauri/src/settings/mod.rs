//! Persisted destination settings and effective destination root resolution.

mod custom_root_validation;
mod effective_root;
mod storage;
mod store;
mod updates;

#[cfg(test)]
mod tests;

pub const DESTINATION_SETTINGS_VERSION: u32 = 1;

pub(crate) use custom_root_validation::validate_custom_root_media_folder;
pub(crate) use store::SettingsStore;
pub(crate) use updates::{
    clear_root_media_folder, learn_home_location, set_force_processed_folder, set_home_location,
    set_rename_in_place, set_root_media_folder,
};
