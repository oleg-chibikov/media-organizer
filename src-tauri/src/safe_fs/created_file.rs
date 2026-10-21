use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

use same_file::Handle;

/// A file this run created with `create_new`, remembered by identity as well as by path.
pub(super) struct CreatedFile {
    path: PathBuf,
    handle: Handle,
}

impl CreatedFile {
    /// Creates `path`, failing with `AlreadyExists` instead of touching an existing file.
    pub(super) fn create_new(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            handle: Handle::from_file(file)?,
        })
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn file_mut(&mut self) -> &mut File {
        self.handle.as_file_mut()
    }

    /// Removes the file only while its path still leads to the file this run created.
    /// A file that was renamed away or replaced by someone else is left alone.
    pub(super) fn discard(self) {
        let still_ours = Handle::from_path(&self.path).is_ok_and(|current| current == self.handle);
        drop(self.handle);
        if still_ours {
            let _ = fs::remove_file(&self.path);
        }
    }
}
