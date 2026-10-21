use std::{fs, path::Path};

use crate::file_ops::files_have_same_bytes;

/// Confirms the destination copy matches the source in size and content.
pub(super) fn verify_destination_matches_source(
    source: &Path,
    destination: &Path,
) -> Result<(), String> {
    if !destination.exists() {
        return Err("Destination copy is missing after copy step.".to_string());
    }

    let source_size = fs::metadata(source)
        .map_err(|error| format!("Failed to read source metadata for verification: {error}"))?
        .len();
    let destination_size = fs::metadata(destination)
        .map_err(|error| format!("Failed to read destination metadata for verification: {error}"))?
        .len();
    if source_size != destination_size {
        return Err(format!(
            "Destination copy size mismatch: source={source_size} bytes, destination={destination_size} bytes."
        ));
    }

    match files_have_same_bytes(source, destination) {
        Ok(true) => Ok(()),
        Ok(false) => Err("Destination copy content mismatch.".to_string()),
        Err(error) => Err(format!(
            "Failed to read files for byte verification: {error}"
        )),
    }
}
