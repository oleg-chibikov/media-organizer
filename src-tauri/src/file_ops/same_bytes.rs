use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
};

const COMPARE_BUFFER_BYTES: usize = 64 * 1024;

/// True when both files have the same size and bytes.
pub(crate) fn files_have_same_bytes(left: &Path, right: &Path) -> io::Result<bool> {
    let len = left.metadata()?.len();
    if len != right.metadata()?.len() {
        return Ok(false);
    }

    let mut left_file = File::open(left)?;
    let mut right_file = File::open(right)?;
    let mut left_buffer = vec![0_u8; COMPARE_BUFFER_BYTES];
    let mut right_buffer = vec![0_u8; COMPARE_BUFFER_BYTES];

    // Videos from one camera often share their opening bytes, so a differing tail saves a full read.
    let tail_start = len.saturating_sub(COMPARE_BUFFER_BYTES as u64);
    if tail_start > 0 {
        left_file.seek(SeekFrom::Start(tail_start))?;
        right_file.seek(SeekFrom::Start(tail_start))?;
        if !chunks_match(
            &mut left_file,
            &mut right_file,
            &mut left_buffer,
            &mut right_buffer,
        )? {
            return Ok(false);
        }
        left_file.rewind()?;
        right_file.rewind()?;
    }

    loop {
        let left_read = fill_buffer(&mut left_file, &mut left_buffer)?;
        let right_read = fill_buffer(&mut right_file, &mut right_buffer)?;
        if left_buffer[..left_read] != right_buffer[..right_read] {
            return Ok(false);
        }
        if left_read == 0 {
            return Ok(true);
        }
    }
}

fn chunks_match(
    left: &mut File,
    right: &mut File,
    left_buffer: &mut [u8],
    right_buffer: &mut [u8],
) -> io::Result<bool> {
    let left_read = fill_buffer(left, left_buffer)?;
    let right_read = fill_buffer(right, right_buffer)?;
    Ok(left_buffer[..left_read] == right_buffer[..right_read])
}

/// Reads until the buffer is full or the file ends; network shares often return short reads.
fn fill_buffer(reader: &mut impl Read, buffer: &mut [u8]) -> io::Result<usize> {
    let mut filled = 0;
    while filled < buffer.len() {
        match reader.read(&mut buffer[filled..]) {
            Ok(0) => break,
            Ok(read) => filled += read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(filled)
}
