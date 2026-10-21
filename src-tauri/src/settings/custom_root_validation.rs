use std::{fs, path::Path};

/// A custom root must be an existing, readable directory.
pub(crate) fn validate_custom_root_media_folder(root: &Path) -> Result<(), String> {
    if !root.exists() {
        return Err(format!(
            "Configured root Media folder does not exist: {}",
            root.display()
        ));
    }

    let metadata = fs::metadata(root).map_err(|error| {
        format!(
            "Configured root Media folder cannot be accessed: {} ({error})",
            root.display()
        )
    })?;

    if !metadata.is_dir() {
        return Err(format!(
            "Configured root Media folder is not a directory: {}",
            root.display()
        ));
    }

    fs::read_dir(root).map_err(|error| {
        format!(
            "Configured root Media folder is not readable: {} ({error})",
            root.display()
        )
    })?;

    Ok(())
}
