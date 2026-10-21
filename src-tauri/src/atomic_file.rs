//! Replaces small app-owned files (settings, history index) so readers never see half a write.

use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

pub(crate) fn write_atomically(path: &Path, contents: &[u8]) -> io::Result<()> {
    let temp_path = path.with_extension("tmp");
    let mut temp_file = File::create(&temp_path)?;
    temp_file.write_all(contents)?;
    temp_file.sync_all()?;
    drop(temp_file);
    fs::rename(&temp_path, path)
}

#[cfg(test)]
mod tests {
    use super::write_atomically;

    #[test]
    fn replaces_existing_contents_and_leaves_no_temp_file() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("index.json");
        std::fs::write(&path, b"old").expect("write old contents");

        write_atomically(&path, b"new").expect("atomic write");

        assert_eq!(std::fs::read(&path).expect("read file"), b"new");
        assert!(!temp.path().join("index.tmp").exists());
    }
}
