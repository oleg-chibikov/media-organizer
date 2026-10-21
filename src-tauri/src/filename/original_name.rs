use super::paren_groups::{
    find_matching_open_paren, is_small_numeric_group, parse_most_nested_group,
};

/// Recovers the camera's original name from `<date> (<original>)[ - device][ (n)].<ext>`,
/// falling back to the stem when no original-name group exists.
pub(crate) fn parse_original_name(file_name: &str) -> String {
    let stem = file_name
        .rsplit_once('.')
        .map(|(without_extension, _)| without_extension)
        .unwrap_or(file_name);
    original_name_from_stem(stem)
}

/// Same as `parse_original_name` for a name that has no extension to drop.
pub(crate) fn original_name_from_stem(stem: &str) -> String {
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

/// Drops a trailing ` (n)` copy counter such as `IMG_1234 (1)`.
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
