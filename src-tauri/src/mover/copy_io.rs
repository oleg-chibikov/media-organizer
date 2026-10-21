use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

const MAX_TEMP_NAME_ATTEMPTS: u32 = 1000;

/// Creates a unique hidden temp file next to `destination` for staging a copy.
pub(super) fn reserve_temp_destination(destination: &Path) -> io::Result<(PathBuf, File)> {
    let parent = destination
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "destination has no parent"))?;
    let file_name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "destination has no file name")
        })?;

    for attempt in 0..MAX_TEMP_NAME_ATTEMPTS {
        let temp_name = format!(
            ".{file_name}.media-organizer-copying-{}-{attempt}.tmp",
            std::process::id()
        );
        let temp_path = parent.join(temp_name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "unable to allocate a unique temporary destination",
    ))
}

pub(super) fn copy_without_overwrite(source: &Path, destination_file: &mut File) -> io::Result<()> {
    let mut source_file = File::open(source)?;
    io::copy(&mut source_file, destination_file)?;
    destination_file.flush()?;
    destination_file.sync_all()?;
    Ok(())
}

/// `create_new` makes publishing fail instead of overwriting a file that appeared meanwhile.
pub(super) fn persist_verified_copy(temp_destination: &Path, destination: &Path) -> io::Result<()> {
    let mut destination_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    copy_without_overwrite(temp_destination, &mut destination_file)
}
