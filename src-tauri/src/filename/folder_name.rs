use super::sanitize_for_windows_filename;

/// Checks a folder name the person may have edited. It must be one plain
/// name: not empty, no path separators, no control characters, not only dots.
pub(crate) fn validated_folder_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("A place folder name is empty.".to_owned());
    }
    if trimmed.contains(['/', '\\']) || trimmed.chars().all(|ch| ch == '.') {
        return Err(format!(
            "\"{trimmed}\" looks like a path. Use a plain folder name."
        ));
    }
    if trimmed.chars().any(char::is_control) {
        return Err(format!(
            "\"{}\" has control characters.",
            trimmed.escape_debug()
        ));
    }
    Ok(sanitize_for_windows_filename(trimmed))
}
