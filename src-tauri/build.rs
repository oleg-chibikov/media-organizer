use std::{
    env,
    fs,
    io,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-changed=binaries/exiftool_files");

    if let Err(error) = copy_exiftool_runtime_files() {
        panic!("Failed to copy ExifTool runtime files: {error}");
    }

    tauri_build::build()
}

fn copy_exiftool_runtime_files() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(io::Error::other)?);
    let source_dir = manifest_dir.join("binaries").join("exiftool_files");

    if !source_dir.exists() {
        return Ok(());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(io::Error::other)?);
    let target_dir = out_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or_else(|| io::Error::other("Could not resolve Cargo target directory"))?;

    let destination_dir = target_dir.join("exiftool_files");
    copy_dir_recursive(&source_dir, &destination_dir)
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &destination_path)?;
        } else {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source_path, &destination_path)?;
        }
    }

    Ok(())
}
