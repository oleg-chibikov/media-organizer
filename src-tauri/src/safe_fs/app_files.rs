use std::{
    fs,
    io::{self, Write},
    path::{Component, Path},
};

use super::{created_file::CreatedFile, occupancy::is_plain_file};

const MAX_TEMP_NAME_ATTEMPTS: u32 = 100;

/// Replaces a small app-owned file (settings, history index) through a fresh temp file,
/// so readers never see half a write. The temp file is created with `create_new`, so a
/// leftover or planted file at that name is never truncated or followed.
pub(crate) fn replace_app_file(path: &Path, contents: &[u8]) -> io::Result<()> {
    let mut temp = reserve_temp_beside(path)?;
    let written = write_synced(&mut temp, contents);
    let replaced = written.and_then(|()| fs::rename(temp.path(), path));
    if replaced.is_err() {
        temp.discard();
    }
    replaced
}

/// Removes an app-owned file only when it is a regular file inside `owned_dir`.
pub(crate) fn remove_app_file(path: &Path, owned_dir: &Path) -> io::Result<()> {
    ensure_inside_dir(path, owned_dir)?;
    if !is_plain_file(path) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("'{}' is not a regular file", path.display()),
        ));
    }
    fs::remove_file(path)
}

/// Fails unless `path` names an entry directly inside the real folder `owned_dir`.
pub(crate) fn ensure_inside_dir(path: &Path, owned_dir: &Path) -> io::Result<()> {
    let outside = || {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("'{}' is outside '{}'", path.display(), owned_dir.display()),
        )
    };
    let Some(Component::Normal(_)) = path.components().next_back() else {
        return Err(outside());
    };
    let parent = path.parent().ok_or_else(outside)?;
    if fs::canonicalize(parent)? != fs::canonicalize(owned_dir)? {
        return Err(outside());
    }
    Ok(())
}

fn write_synced(temp: &mut CreatedFile, contents: &[u8]) -> io::Result<()> {
    let file = temp.file_mut();
    file.write_all(contents)?;
    file.flush()?;
    file.sync_all()
}

fn reserve_temp_beside(path: &Path) -> io::Result<CreatedFile> {
    let file_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no file name"))?
        .to_string_lossy();
    for attempt in 0..MAX_TEMP_NAME_ATTEMPTS {
        let temp_name = format!(".{file_name}.{}-{attempt}.tmp", std::process::id());
        match CreatedFile::create_new(&path.with_file_name(temp_name)) {
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            reserved => return reserved,
        }
    }
    Err(io::Error::other("unable to allocate a unique temp file"))
}
