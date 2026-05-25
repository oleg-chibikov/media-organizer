pub fn parse_original_name(file_name: &str) -> String {
    let stem = file_name
        .rsplit_once('.')
        .map(|(without_extension, _)| without_extension)
        .unwrap_or(file_name);

    let stem_without_duplicate_suffix = strip_trailing_duplicate_suffix(stem);
    let end = stem_without_duplicate_suffix.rfind(')');
    let start =
        end.and_then(|index| find_matching_open_paren(stem_without_duplicate_suffix, index));

    match (start, end) {
        (Some(s), Some(e)) if s < e => {
            let inner = stem_without_duplicate_suffix[s + 1..e].trim();
            if !inner.is_empty() && !is_small_numeric_group(inner) {
                parse_most_nested_group(inner).unwrap_or_else(|| inner.to_string())
            } else {
                stem_without_duplicate_suffix.to_string()
            }
        }
        _ => stem_without_duplicate_suffix.to_string(),
    }
}

fn find_matching_open_paren(value: &str, close_index: usize) -> Option<usize> {
    let bytes = value.as_bytes();
    if close_index >= bytes.len() || bytes[close_index] != b')' {
        return None;
    }

    let mut depth = 0_u32;
    for index in (0..=close_index).rev() {
        if bytes[index] == b')' {
            depth += 1;
            continue;
        }
        if bytes[index] == b'(' {
            if depth == 0 {
                return None;
            }
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn parse_pure_number(value: &str) -> Option<u32> {
    if value.is_empty() || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }

    value.parse::<u32>().ok()
}

fn is_small_numeric_group(value: &str) -> bool {
    parse_pure_number(value).is_some_and(|number| number < 100)
}

fn parse_most_nested_group(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut stack: Vec<usize> = Vec::new();
    let mut best: Option<(usize, usize, String)> = None;

    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'(' {
            stack.push(index);
            continue;
        }

        if *byte != b')' {
            continue;
        }

        let Some(start) = stack.pop() else {
            continue;
        };

        let depth = stack.len() + 1;
        let inner = value[start + 1..index].trim();

        if inner.is_empty() || is_small_numeric_group(inner) {
            continue;
        }

        match &best {
            Some((best_depth, best_index, _))
                if depth < *best_depth || (depth == *best_depth && index <= *best_index) => {}
            _ => {
                best = Some((depth, index, inner.to_string()));
            }
        }
    }

    best.map(|(_, _, inner)| inner)
}

fn strip_trailing_duplicate_suffix(stem: &str) -> &str {
    let Some(stripped) = stem.strip_suffix(')') else {
        return stem;
    };
    let Some(start) = stripped.rfind('(') else {
        return stem;
    };
    let inner = stripped[start + 1..].trim();
    let has_whitespace_before_group = start > 0 && stripped[..start].ends_with(char::is_whitespace);

    if has_whitespace_before_group && is_small_numeric_group(inner) {
        stripped[..start].trim_end()
    } else {
        stem
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
            "PXL_20241102_020933409.LS"
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
    fn parses_parenthesized_original_name_with_device_suffix_after_group() {
        assert_eq!(
            parse_original_name("2026-01-01 09-45-05 (20260101_094504) - SM-G991B.jpg"),
            "20260101_094504"
        );
    }

    #[test]
    fn parses_parenthesized_original_name_with_nested_parentheses_and_device_suffix() {
        assert_eq!(
            parse_original_name("2025-03-02 15-15-16 (20250302_151515(0)) - SM-G991B.jpg"),
            "20250302_151515(0)"
        );
    }

    #[test]
    fn parses_most_nested_parenthesized_original_name() {
        assert_eq!(
            parse_original_name(
                "2024-10-20 18-14-18 (2024-10-20 18-14-20 (PXL_20241020_071418278)) - Pixel 7 Pro.jpg"
            ),
            "PXL_20241020_071418278"
        );
    }

    #[test]
    fn sanitizes_invalid_windows_characters() {
        assert_eq!(sanitize_for_windows_filename("A<B>C:foo?"), "A_B_C_foo_");
        assert_eq!(sanitize_for_windows_filename("name. "), "name");
    }
}
