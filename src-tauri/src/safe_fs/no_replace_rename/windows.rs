use std::{
    ffi::OsString,
    io,
    os::windows::ffi::OsStrExt,
    path::{Component, Path, PathBuf, Prefix},
};

use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_WRITE_THROUGH};

/// Mirrors std, which only adds the `\\?\` prefix once a path nears `MAX_PATH`.
const LONG_PATH_THRESHOLD: usize = 248;
const ERROR_INVALID_FUNCTION: i32 = 1;
const ERROR_NOT_SUPPORTED: i32 = 50;

/// Without `MOVEFILE_REPLACE_EXISTING` the move fails when the destination exists, and
/// without `MOVEFILE_COPY_ALLOWED` it fails across volumes instead of copying.
pub(super) fn rename_exclusive(source: &Path, destination: &Path) -> io::Result<()> {
    let source = wide_path(source)?;
    let destination = wide_path(destination)?;
    // SAFETY: both pointers are valid NUL-terminated UTF-16 strings that outlive the call.
    let moved = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_WRITE_THROUGH,
        )
    };
    if moved != 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

pub(super) fn means_unsupported(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::Unsupported
        || matches!(
            error.raw_os_error(),
            Some(ERROR_INVALID_FUNCTION | ERROR_NOT_SUPPORTED)
        )
}

fn wide_path(path: &Path) -> io::Result<Vec<u16>> {
    let mut wide: Vec<u16> = long_path_form(path).as_os_str().encode_wide().collect();
    if wide.contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "path contains a NUL character",
        ));
    }
    wide.push(0);
    Ok(wide)
}

/// Rewrites long absolute disk and UNC paths into their verbatim form.
fn long_path_form(path: &Path) -> PathBuf {
    if path.as_os_str().encode_wide().count() < LONG_PATH_THRESHOLD {
        return path.to_path_buf();
    }
    let mut components = path.components();
    let mut verbatim = match components.next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Disk(letter) => OsString::from(format!(r"\\?\{}:", letter as char)),
            Prefix::UNC(server, share) => {
                let mut unc = OsString::from(r"\\?\UNC\");
                unc.push(server);
                unc.push(r"\");
                unc.push(share);
                unc
            }
            _ => return path.to_path_buf(),
        },
        _ => return path.to_path_buf(),
    };
    for component in components {
        match component {
            Component::Normal(name) => {
                verbatim.push(r"\");
                verbatim.push(name);
            }
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) => return path.to_path_buf(),
        }
    }
    PathBuf::from(verbatim)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::long_path_form;

    #[test]
    fn prefixes_long_disk_and_unc_paths_only() {
        let long_name = "a".repeat(260);
        let disk = format!(r"C:\media\{long_name}.jpg");
        let unc = format!(r"\\nas\share\{long_name}.jpg");

        assert_eq!(
            long_path_form(Path::new(r"C:\a.jpg")),
            Path::new(r"C:\a.jpg")
        );
        assert_eq!(
            long_path_form(Path::new(&disk)),
            Path::new(&format!(r"\\?\{disk}"))
        );
        assert_eq!(
            long_path_form(Path::new(&unc)),
            Path::new(&format!(r"\\?\UNC\nas\share\{long_name}.jpg"))
        );
    }
}
