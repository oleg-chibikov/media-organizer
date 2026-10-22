//! Opens a file in the app the platform picks for its type.

use std::path::Path;
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use std::process::Command;

pub(crate) fn open_path_in_default_app(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err("File no longer exists on disk.".to_string());
    }

    #[cfg(target_os = "windows")]
    let program = "explorer";
    #[cfg(target_os = "macos")]
    let program = "open";
    #[cfg(target_os = "linux")]
    let program = "xdg-open";

    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
        Command::new(program)
            .arg(path)
            .spawn()
            .map_err(|error| format!("Failed to open the file: {error}"))?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err("Opening files is not supported on this platform.".to_string())
}
