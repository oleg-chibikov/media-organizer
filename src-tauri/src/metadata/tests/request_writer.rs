use std::path::PathBuf;

use crate::metadata::request_writer::{write_request, EXIFTOOL_TAGS};

#[test]
fn writes_only_read_options_and_the_requested_path() {
    let photo = std::env::temp_dir().join("photo.jpg");
    let mut written = Vec::new();

    write_request(&mut written, std::slice::from_ref(&photo), 7).expect("write request");

    let text = String::from_utf8(written).expect("utf-8 request");
    let mut allowed: Vec<&str> = vec!["-json", "-execute7", photo.to_str().expect("utf-8")];
    allowed.extend(EXIFTOOL_TAGS);
    for line in text.lines() {
        assert!(
            allowed.contains(&line),
            "unexpected ExifTool argument {line:?}"
        );
    }
    for tag in EXIFTOOL_TAGS {
        assert!(!tag.contains(['=', '<', '>']), "{tag} would write metadata");
    }
}

#[test]
fn refuses_paths_that_could_inject_arguments() {
    let folder = std::env::temp_dir();
    let unsafe_paths = [
        folder.join("a.jpg\n-overwrite_original\n-all="),
        folder.join("b.jpg\r-o"),
        PathBuf::from("-all="),
        PathBuf::from("relative.jpg"),
    ];

    for unsafe_path in unsafe_paths {
        let mut written = Vec::new();
        assert!(write_request(&mut written, std::slice::from_ref(&unsafe_path), 1).is_err());
        assert!(written.is_empty(), "{unsafe_path:?} reached ExifTool");
    }
}
