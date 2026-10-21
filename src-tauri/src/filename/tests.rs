use crate::filename::{parse_original_name, sanitize_for_windows_filename};

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
