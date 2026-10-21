use std::{fs, fs::File, io::Read, path::Path};

const VERIFY_BUFFER_BYTES: usize = 64 * 1024;

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

    let mut source_file = File::open(source)
        .map_err(|error| format!("Failed to open source for byte verification: {error}"))?;
    let mut destination_file = File::open(destination)
        .map_err(|error| format!("Failed to open destination for byte verification: {error}"))?;
    let mut source_buffer = [0_u8; VERIFY_BUFFER_BYTES];
    let mut destination_buffer = [0_u8; VERIFY_BUFFER_BYTES];

    loop {
        let source_read = source_file
            .read(&mut source_buffer)
            .map_err(|error| format!("Failed to read source for byte verification: {error}"))?;
        let destination_read = destination_file
            .read(&mut destination_buffer)
            .map_err(|error| {
                format!("Failed to read destination for byte verification: {error}")
            })?;

        if source_read != destination_read
            || source_buffer[..source_read] != destination_buffer[..destination_read]
        {
            return Err("Destination copy content mismatch.".to_string());
        }

        if source_read == 0 {
            break;
        }
    }

    Ok(())
}
