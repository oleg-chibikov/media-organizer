//! Reveals a file in the platform file manager (Explorer, Finder or a freedesktop manager).

mod open;
#[cfg(test)]
mod tests;

pub(crate) use open::open_path_in_default_app;

use std::path::Path;
#[cfg(target_os = "windows")]
use std::path::PathBuf;
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use std::process::Command;

pub(crate) fn reveal_path_in_system_explorer(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err("File no longer exists on disk.".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let target = path
            .canonicalize()
            .map_err(|error| format!("Failed to resolve file path: {error}"))?;
        Command::new("explorer")
            .arg("/select,")
            .arg(strip_windows_verbatim_prefix(&target))
            .spawn()
            .map_err(|error| format!("Failed to open File Explorer: {error}"))?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .map_err(|error| format!("Failed to open Finder: {error}"))?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let parent = path
            .parent()
            .ok_or_else(|| "File has no parent directory.".to_string())?;

        let attempted_select = Command::new("dbus-send")
            .arg("--session")
            .arg("--dest=org.freedesktop.FileManager1")
            .arg("--type=method_call")
            .arg("/org/freedesktop/FileManager1")
            .arg("org.freedesktop.FileManager1.ShowItems")
            .arg(format!(
                "array:string:file://{}",
                path.to_string_lossy().replace(' ', "%20")
            ))
            .arg("string:")
            .spawn();

        if attempted_select.is_ok() {
            return Ok(());
        }

        Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|error| format!("Failed to open file manager: {error}"))?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err("Reveal in system explorer is not supported on this platform.".to_string())
}

/// Explorer cannot select `\\?\` verbatim paths returned by `canonicalize`.
#[cfg(target_os = "windows")]
fn strip_windows_verbatim_prefix(path: &Path) -> PathBuf {
    let path_value = path.to_string_lossy();

    if let Some(stripped) = path_value.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{stripped}"));
    }

    if let Some(stripped) = path_value.strip_prefix(r"\\?\") {
        return PathBuf::from(stripped);
    }

    path.to_path_buf()
}
