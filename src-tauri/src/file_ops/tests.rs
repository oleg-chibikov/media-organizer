use std::fs;

use tempfile::tempdir;

use super::{files_have_same_bytes, files_have_same_bytes_cached};

#[test]
fn compares_files_by_size_and_bytes() {
    let temp = tempdir().expect("temp dir");
    let original = temp.path().join("original.jpg");
    let same = temp.path().join("same.jpg");
    let edited = temp.path().join("edited.jpg");
    let longer = temp.path().join("longer.jpg");
    fs::write(&original, b"image-bytes").expect("write original");
    fs::write(&same, b"image-bytes").expect("write same");
    fs::write(&edited, b"image-BYTES").expect("write edited");
    fs::write(&longer, b"image-bytes!").expect("write longer");

    assert!(files_have_same_bytes(&original, &same).expect("compare same"));
    assert!(!files_have_same_bytes(&original, &edited).expect("compare edited"));
    assert!(!files_have_same_bytes(&original, &longer).expect("compare longer"));
}

#[test]
fn finds_a_difference_past_the_first_chunk() {
    let temp = tempdir().expect("temp dir");
    let original = temp.path().join("original.mp4");
    let edited = temp.path().join("edited.mp4");
    let bytes = vec![7_u8; 200 * 1024];
    let mut edited_bytes = bytes.clone();
    edited_bytes[100 * 1024] = 8;
    fs::write(&original, &bytes).expect("write original");
    fs::write(&edited, &edited_bytes).expect("write edited");

    assert!(files_have_same_bytes(&original, &original).expect("compare same"));
    assert!(!files_have_same_bytes(&original, &edited).expect("compare middle"));
    edited_bytes[100 * 1024] = 7;
    *edited_bytes.last_mut().expect("last byte") = 8;
    fs::write(&edited, &edited_bytes).expect("rewrite edited");
    assert!(!files_have_same_bytes(&original, &edited).expect("compare tail"));
}

#[test]
fn cached_comparison_notices_a_changed_file() {
    let temp = tempdir().expect("temp dir");
    let original = temp.path().join("original.jpg");
    let copy = temp.path().join("copy.jpg");
    fs::write(&original, b"image-bytes").expect("write original");
    fs::write(&copy, b"image-bytes").expect("write copy");
    assert!(files_have_same_bytes_cached(&original, &copy).expect("compare same"));

    fs::write(&copy, b"other-bytes!").expect("rewrite copy");

    assert!(!files_have_same_bytes_cached(&original, &copy).expect("compare changed"));
}
