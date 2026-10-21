use std::path::Path;

use crate::{path_safety, safe_fs, settings::SettingsStore};

/// Removes the empty folders a finished run left in the selected folder, the folder included.
/// The destination root and everything inside it stay, so `Processed` and in-place runs keep
/// their folders. Returns whether the selected folder itself was removed.
pub(crate) fn remove_empty_source_folders(
    settings: &SettingsStore,
    selected_folder: &Path,
) -> Result<bool, String> {
    if !safe_fs::path_is_occupied(selected_folder) {
        return Ok(false);
    }
    path_safety::validate_selected_folder(selected_folder)?;
    let destination = settings.effective_destination_root(selected_folder)?;
    Ok(safe_fs::remove_empty_folders(
        selected_folder,
        Path::new(&destination.root_path),
    ))
}
