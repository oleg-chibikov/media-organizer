use std::{fs::File, io::Read, path::Path};

const COMPARE_BUFFER_BYTES: usize = 64 * 1024;

/// True when both files have the same size and bytes.
pub(super) fn files_have_identical_contents(
    source_path: &Path,
    destination_path: &Path,
) -> Result<bool, String> {
    let source_meta = source_path
        .metadata()
        .map_err(|error| format!("Failed to read source metadata for conflict check: {error}"))?;
    let destination_meta = destination_path.metadata().map_err(|error| {
        format!("Failed to read destination metadata for conflict check: {error}")
    })?;

    if source_meta.len() != destination_meta.len() {
        return Ok(false);
    }

    let mut source_file = File::open(source_path)
        .map_err(|error| format!("Failed to open source for conflict check: {error}"))?;
    let mut destination_file = File::open(destination_path)
        .map_err(|error| format!("Failed to open destination for conflict check: {error}"))?;
    let mut source_buffer = [0_u8; COMPARE_BUFFER_BYTES];
    let mut destination_buffer = [0_u8; COMPARE_BUFFER_BYTES];

    loop {
        let source_read = source_file
            .read(&mut source_buffer)
            .map_err(|error| format!("Failed reading source for conflict check: {error}"))?;
        let destination_read = destination_file
            .read(&mut destination_buffer)
            .map_err(|error| format!("Failed reading destination for conflict check: {error}"))?;

        if source_read != destination_read
            || source_buffer[..source_read] != destination_buffer[..destination_read]
        {
            return Ok(false);
        }

        if source_read == 0 {
            break;
        }
    }

    Ok(true)
}
