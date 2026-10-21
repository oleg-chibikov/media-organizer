use std::{io, path::Path};

pub(super) fn rename_exclusive(_source: &Path, _destination: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "no exclusive rename on this platform",
    ))
}

pub(super) fn means_unsupported(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::Unsupported
}
