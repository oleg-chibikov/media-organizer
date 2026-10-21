const INVALID_WINDOWS_CHARS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// Replaces reserved characters and trailing dots/spaces that Windows rejects.
pub(crate) fn sanitize_for_windows_filename(input: &str) -> String {
    let mut sanitized: String = input
        .chars()
        .map(|ch| {
            if INVALID_WINDOWS_CHARS.contains(&ch) {
                '_'
            } else {
                ch
            }
        })
        .collect();
    while sanitized.ends_with('.') || sanitized.ends_with(' ') {
        sanitized.pop();
    }
    if sanitized.is_empty() {
        "_".to_string()
    } else {
        sanitized
    }
}
