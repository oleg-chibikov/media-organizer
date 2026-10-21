use std::{io, path::Path};

use super::{is_unsupported_errno, nul_terminated};

/// Calls `renameat2` through `syscall` so it works with glibc and musl alike.
pub(super) fn rename_exclusive(source: &Path, destination: &Path) -> io::Result<()> {
    let source = nul_terminated(source)?;
    let destination = nul_terminated(destination)?;
    // SAFETY: both pointers are valid NUL-terminated strings that outlive the call.
    let status = unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if status == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

pub(super) fn means_unsupported(error: &io::Error) -> bool {
    is_unsupported_errno(error)
}
