pub fn parse_original_name(stem: &str) -> String {
    let start = stem.rfind('(');
    let end = stem.rfind(')');

    match (start, end) {
        (Some(s), Some(e)) if s < e => {
            let inner = stem[s + 1..e].trim();
            if inner.is_empty() {
                stem.to_string()
            } else {
                inner.to_string()
            }
        }
        _ => stem.to_string(),
    }
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
        let input = "2024-11-02 13-09-41 (PXL_20241102_020933409.LS)";
        assert_eq!(parse_original_name(input), "PXL_20241102_020933409.LS");
    }

    #[test]
    fn falls_back_to_full_stem_without_parentheses() {
        assert_eq!(
            parse_original_name("PXL_20260305_072017616.LS"),
            "PXL_20260305_072017616.LS"
        );
    }

    #[test]
    fn handles_multiple_parenthesized_segments_using_last_group() {
        assert_eq!(
            parse_original_name("2026-03-05 07-20-17 (tmp) (IMG_1234)"),
            "IMG_1234"
        );
    }

    #[test]
    fn sanitizes_invalid_windows_characters() {
        assert_eq!(sanitize_for_windows_filename("A<B>C:foo?"), "A_B_C_foo_");
        assert_eq!(sanitize_for_windows_filename("name. "), "name");
    }
}
