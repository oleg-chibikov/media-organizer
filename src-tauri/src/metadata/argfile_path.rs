use std::path::Path;

/// Turns a source path into one ExifTool argument line.
///
/// Workers read arguments one per line from `-@ -`, so a path holding a line break could
/// smuggle in write options such as `-overwrite_original` or `-TAG=`. Only absolute,
/// single-line Unicode paths pass, which also rules out a leading `-` or `#`.
pub(crate) fn exiftool_path_argument(path: &Path) -> Result<&str, String> {
    let text = path.to_str().ok_or_else(|| {
        "Path is not valid Unicode, so ExifTool can't read it safely.".to_string()
    })?;
    if !path.is_absolute() {
        return Err("Path is not absolute, so ExifTool can't read it safely.".to_string());
    }
    if text.contains(['\n', '\r', '\0']) {
        return Err("Path contains a line break, so ExifTool can't read it safely.".to_string());
    }
    Ok(text)
}
