mod app_files;
mod move_file;
mod removal;
mod staged_copy;

use std::{fs, path::Path};

/// Names in `directory`, sorted, so tests can prove no temp file was left behind.
fn entry_names(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(directory)
        .expect("read dir")
        .map(|entry| {
            entry
                .expect("dir entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

fn write(path: &Path, contents: &[u8]) {
    fs::write(path, contents).expect("write fixture");
}

fn read(path: &Path) -> Vec<u8> {
    fs::read(path).expect("read fixture")
}
