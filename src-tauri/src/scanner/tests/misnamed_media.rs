use std::path::Path;

use crate::models::{FileKind, FileRecord, ScanStatus};
use crate::scanner::{file_signature::media_extension_from_header, scanned_file::record_for_path};

const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE1, 0, 0];
const MP4: &[u8] = b"\0\0\0\x20ftypisom\0\0\x02\0";

fn ftyp(brand: &[u8; 4]) -> Vec<u8> {
    [b"\0\0\0\x18ftyp".as_slice(), brand].concat()
}

fn transport_stream(prefix: usize, packet: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; 200];
    bytes[prefix] = 0x47;
    bytes[prefix + packet] = 0x47;
    bytes
}

fn scan_one(name: &str, content: &[u8]) -> FileRecord {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join(name);
    std::fs::write(&path, content).expect("write file");
    record_for_path(temp.path(), &path)
}

#[test]
fn names_media_extensions_from_file_headers() {
    let cases: [(Vec<u8>, Option<&str>); 11] = [
        (JPEG.to_vec(), Some("jpg")),
        (b"\x89PNG\r\n\x1a\n".to_vec(), Some("png")),
        (b"RIFF\0\0\0\0WEBPVP8 ".to_vec(), Some("webp")),
        (b"RIFF\0\0\0\0AVI LIST".to_vec(), Some("avi")),
        (ftyp(b"heic"), Some("heic")),
        (ftyp(b"qt  "), Some("mov")),
        (MP4.to_vec(), Some("mp4")),
        (transport_stream(0, 188), Some("mts")),
        (transport_stream(4, 192), Some("m2ts")),
        (ftyp(b"M4A "), None),
        (b"II*\0\x08\0\0\0".to_vec(), None),
    ];
    for (header, expected) in cases {
        assert_eq!(media_extension_from_header(&header), expected, "{header:?}");
    }
}

#[test]
fn finds_a_photo_hidden_behind_a_broken_extension() {
    let record = scan_one("name(Файл1.jpg)", JPEG);

    assert!(matches!(record.scan_status, ScanStatus::Discovered));
    assert!(matches!(record.kind, FileKind::Photo));
    assert_eq!(record.extension, "jpg");
    assert_eq!(record.original_name, "Файл1");
}

#[test]
fn finds_a_video_without_an_extension() {
    let record = scan_one("IMG_1234", MP4);

    assert!(matches!(record.kind, FileKind::Video));
    assert_eq!(record.extension, "mp4");
    assert_eq!(record.original_name, "IMG_1234");
}

#[test]
fn trusts_real_extensions_and_skips_unrecognized_content() {
    let sidecar = scan_one("GOPR0001.THM", JPEG);
    let text = scan_one("notes(1.jpg)", b"plain text");

    assert!(matches!(
        sidecar.scan_status,
        ScanStatus::SkippedUnsupported
    ));
    assert!(matches!(text.scan_status, ScanStatus::SkippedUnsupported));
    assert_eq!(text.extension, "jpg)");
    assert!(Path::new(&text.source_path).ends_with("notes(1.jpg)"));
}
