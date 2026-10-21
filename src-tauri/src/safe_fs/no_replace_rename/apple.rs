use std::{io, path::Path};

use super::{is_unsupported_errno, nul_terminated};

pub(super) fn rename_exclusive(source: &Path, destination: &Path) -> io::Result<()> {
    let source = nul_terminated(source)?;
    let destination = nul_terminated(destination)?;
    // SAFETY: both pointers are valid NUL-terminated strings that outlive the call.
    let status =
        unsafe { libc::renamex_np(source.as_ptr(), destination.as_ptr(), libc::RENAME_EXCL) };
    if status == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

pub(super) fn means_unsupported(error: &io::Error) -> bool {
    is_unsupported_errno(error)
}
