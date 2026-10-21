//! An atomic rename that fails with `AlreadyExists` instead of replacing the destination.
//! `std::fs::rename` silently replaces on every platform, so it can't close the race
//! between a conflict check and the move.

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod platform;
#[cfg(target_vendor = "apple")]
#[path = "apple.rs"]
mod platform;
#[cfg(windows)]
#[path = "windows.rs"]
mod platform;
#[cfg(not(any(target_os = "linux", target_vendor = "apple", windows)))]
#[path = "unsupported.rs"]
mod platform;

use std::{io, path::Path};

/// Renames `source` to `destination` only while `destination` is free.
/// Filesystems without an exclusive rename report `ErrorKind::Unsupported`.
pub(crate) fn rename_no_replace(source: &Path, destination: &Path) -> io::Result<()> {
    platform::rename_exclusive(source, destination).map_err(|error| {
        if platform::means_unsupported(&error) {
            io::Error::new(
                io::ErrorKind::Unsupported,
                format!("no-overwrite rename isn't supported here: {error}"),
            )
        } else {
            error
        }
    })
}

#[cfg(any(target_os = "linux", target_vendor = "apple"))]
fn nul_terminated(path: &Path) -> io::Result<std::ffi::CString> {
    use std::os::unix::ffi::OsStrExt;

    std::ffi::CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contains a NUL byte"))
}

#[cfg(any(target_os = "linux", target_vendor = "apple"))]
fn is_unsupported_errno(error: &io::Error) -> bool {
    let unsupported = [libc::EINVAL, libc::ENOSYS, libc::ENOTSUP, libc::EOPNOTSUPP];
    error.kind() == io::ErrorKind::Unsupported
        || error
            .raw_os_error()
            .is_some_and(|code| unsupported.contains(&code))
}
