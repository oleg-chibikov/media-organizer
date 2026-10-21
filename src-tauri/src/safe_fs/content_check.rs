use std::{fs, io, path::Path};

use crate::file_ops::files_have_same_bytes;

/// Confirms `copy` holds exactly the bytes of `original`; mismatches are `InvalidData`.
pub(crate) fn verify_same_content(original: &Path, copy: &Path) -> io::Result<()> {
    let original_size = fs::metadata(original)?.len();
    let copy_size = fs::metadata(copy)?.len();
    if original_size != copy_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("size mismatch: original={original_size} bytes, copy={copy_size} bytes"),
        ));
    }
    if !files_have_same_bytes(original, copy)? {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "content mismatch",
        ));
    }
    Ok(())
}
