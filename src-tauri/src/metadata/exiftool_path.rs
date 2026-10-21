use std::path::{Path, PathBuf};

const SIDECAR_FILE_NAME: &str = "exiftool-x86_64-pc-windows-msvc.exe";
/// Tauri bundles the sidecar as `exiftool.exe` on Windows and `exiftool` elsewhere.
const BUNDLED_FILE_NAME: &str = if cfg!(windows) {
    "exiftool.exe"
} else {
    "exiftool"
};

/// Finds the bundled ExifTool sidecar in resource, executable and dev source locations.
pub(super) fn resolve_exiftool_path(resource_dir: Option<&Path>) -> Result<PathBuf, String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(resource_dir) = resource_dir {
        candidates.push(resource_dir.join(SIDECAR_FILE_NAME));
        candidates.push(resource_dir.join(BUNDLED_FILE_NAME));
        candidates.push(resource_dir.join("binaries").join(SIDECAR_FILE_NAME));
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join(SIDECAR_FILE_NAME));
            candidates.push(exe_dir.join(BUNDLED_FILE_NAME));
        }
    }

    // Only dev builds look in the working directory; a release must not run whatever
    // executable happens to sit under the folder it was started from.
    if cfg!(debug_assertions) {
        let current_dir = std::env::current_dir()
            .map_err(|error| format!("Failed to resolve current directory: {error}"))?;
        candidates.push(
            current_dir
                .join("src-tauri")
                .join("binaries")
                .join(SIDECAR_FILE_NAME),
        );
    }

    if let Some(found) = candidates.into_iter().find(|path| path.exists()) {
        return Ok(found);
    }

    Err(
        "Missing ExifTool sidecar binary. Expected `src-tauri/binaries/exiftool-x86_64-pc-windows-msvc.exe`."
            .to_string(),
    )
}
