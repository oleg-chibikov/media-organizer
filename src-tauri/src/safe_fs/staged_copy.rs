use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

use super::{
    content_check::verify_same_content, created_file::CreatedFile, move_file::move_file_no_replace,
    occupancy::path_is_occupied,
};

const MAX_TEMP_NAME_ATTEMPTS: u32 = 1000;

/// Copies `source` to a free `destination` and keeps the source.
///
/// The bytes land in a hidden temp file beside the destination, get synced and compared with
/// the source, and only then take the destination name through a no-replace move. A
/// destination that appears meanwhile fails the copy with `AlreadyExists`.
pub(crate) fn copy_verified_no_replace(source: &Path, destination: &Path) -> io::Result<()> {
    copy_verified_with_publisher(source, destination, move_file_no_replace)
}

pub(super) fn copy_verified_with_publisher(
    source: &Path,
    destination: &Path,
    publish: impl Fn(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
    if path_is_occupied(destination) {
        return Err(io::Error::from(io::ErrorKind::AlreadyExists));
    }
    let mut staged = reserve_temp_beside(destination)?;
    let published = write_verified(source, &mut staged).and_then(|()| {
        match publish(staged.path(), destination) {
            Err(error) if error.kind() == io::ErrorKind::Unsupported => {
                copy_to_new_file(staged.path(), destination)
            }
            result => result,
        }
    });
    staged.discard();
    published?;
    verify_same_content(source, destination)
}

/// Last-resort publish for filesystems with neither exclusive rename nor hard links.
fn copy_to_new_file(source: &Path, destination: &Path) -> io::Result<()> {
    let mut created = CreatedFile::create_new(destination)?;
    let written = write_verified(source, &mut created);
    if written.is_err() {
        created.discard();
    }
    written
}

fn write_verified(source: &Path, target: &mut CreatedFile) -> io::Result<()> {
    let mut source_file = File::open(source)?;
    let file = target.file_mut();
    io::copy(&mut source_file, file)?;
    file.flush()?;
    file.sync_all()?;
    verify_same_content(source, target.path())
}

fn reserve_temp_beside(destination: &Path) -> io::Result<CreatedFile> {
    let invalid = |message| io::Error::new(io::ErrorKind::InvalidInput, message);
    let parent = destination
        .parent()
        .ok_or_else(|| invalid("destination has no parent"))?;
    let file_name = destination
        .file_name()
        .ok_or_else(|| invalid("destination has no file name"))?
        .to_string_lossy();

    for attempt in 0..MAX_TEMP_NAME_ATTEMPTS {
        let temp_name = format!(
            ".{file_name}.media-organizer-copying-{}-{attempt}.tmp",
            std::process::id()
        );
        match CreatedFile::create_new(&parent.join(temp_name)) {
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            reserved => return reserved,
        }
    }
    Err(io::Error::other(
        "unable to allocate a unique temporary copy path",
    ))
}
