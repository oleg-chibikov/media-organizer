pub fn parse_original_name(file_name: &str) -> String {
    let stem = file_name
        .rsplit_once('.')
        .map(|(without_extension, _)| without_extension)
        .unwrap_or(file_name);

    let start = stem.rfind('(');
    let end = stem.rfind(')');

    match (start, end) {
        (Some(s), Some(e)) if s < e && e == stem.len() - 1 => {
            let inner = stem[s + 1..e].trim();
            let has_whitespace_before_group = s > 0 && stem[..s].ends_with(char::is_whitespace);

            // Ignore trailing duplicate-maker suffixes like " (1)" and " (2)".
            if has_whitespace_before_group && is_duplicate_suffix(inner) {
                stem[..s].trim_end().to_string()
            } else if has_whitespace_before_group && !inner.is_empty() {
                inner.to_string()
            } else {
                stem.to_string()
            }
        }
        _ => stem.to_string(),
    }
}

fn is_duplicate_suffix(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit())
}

pub fn sanitize_for_windows_filename(input: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut sanitized: String = input
        .chars()
        .map(|ch| if invalid_chars.contains(&ch) { '_' } else { ch })
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

#[cfg(test)]
mod tests {
    use super::{parse_original_name, sanitize_for_windows_filename};

    #[test]
    fn parses_parenthesized_original_name() {
        let input = "2024-11-02 13-09-41 (PXL_20241102_020933409.LS).mp4";
        assert_eq!(parse_original_name(input), "PXL_20241102_020933409.LS");
    }

    #[test]
    fn falls_back_to_full_stem_without_parentheses() {
        assert_eq!(
            parse_original_name("PXL_20260305_072017616.LS.mp4"),
            "PXL_20260305_072017616.LS"
        );
    }

    #[test]
    fn handles_multiple_parenthesized_segments_using_last_group() {
        assert_eq!(
            parse_original_name("2026-03-05 07-20-17 (tmp) (IMG_1234).mp4"),
            "IMG_1234"
        );
    }

    #[test]
    fn ignores_trailing_duplicate_suffix_group() {
        assert_eq!(parse_original_name("IMG_1234 (1).jpg"), "IMG_1234");
    }

    #[test]
    fn keeps_embedded_original_name_when_duplicate_suffix_exists() {
        assert_eq!(
            parse_original_name("2024-11-02 13-09-41 (PXL_20241102_020933409.LS) (2).mp4"),
            "2024-11-02 13-09-41 (PXL_20241102_020933409.LS)"
        );
    }

    #[test]
    fn keeps_full_stem_for_parenthesized_number_without_space() {
        assert_eq!(
            parse_original_name("120260101_094503(0).jpg"),
            "120260101_094503(0)"
        );
    }

    #[test]
    fn sanitizes_invalid_windows_characters() {
        assert_eq!(sanitize_for_windows_filename("A<B>C:foo?"), "A_B_C_foo_");
        assert_eq!(sanitize_for_windows_filename("name. "), "name");
    }
}
