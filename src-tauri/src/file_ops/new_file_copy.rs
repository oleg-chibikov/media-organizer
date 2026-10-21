use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::Path,
};

/// Copies `source` to a file that must not exist yet, removing the partial copy on failure.
pub(crate) fn copy_into_new_file(source: &Path, destination: &Path) -> io::Result<()> {
    let mut destination_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    copy_into(source, &mut destination_file).inspect_err(|_| {
        drop(destination_file);
        let _ = fs::remove_file(destination);
    })
}

/// Streams `source` into an already opened file and flushes it to disk.
pub(crate) fn copy_into(source: &Path, destination_file: &mut File) -> io::Result<()> {
    let mut source_file = File::open(source)?;
    io::copy(&mut source_file, destination_file)?;
    destination_file.flush()?;
    destination_file.sync_all()
}
