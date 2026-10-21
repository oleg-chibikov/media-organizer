use std::fs;

use tempfile::tempdir;

use super::files_have_same_bytes;

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
