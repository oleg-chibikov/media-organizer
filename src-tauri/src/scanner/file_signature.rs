use std::{fs::File, io::Read, path::Path};

const HEADER_LENGTH: u64 = 200;
const TS_PACKET: usize = 188;
const M2TS_PACKET: usize = 192;
const HEIC_BRANDS: &[&[u8]] = &[b"heic", b"heix", b"heim", b"heis", b"hevc", b"hevx"];
const HEIF_BRANDS: &[&[u8]] = &[b"mif1", b"msf1"];
const MOV_BRANDS: &[&[u8]] = &[b"qt  "];
const M4V_BRANDS: &[&[u8]] = &[b"M4V ", b"M4VH", b"M4VP"];
const MP4_BRANDS: &[&[u8]] = &[
    b"isom", b"iso2", b"iso4", b"iso5", b"iso6", b"mp41", b"mp42", b"avc1", b"mmp4", b"MSNV",
    b"XAVC",
];

/// Reads the first bytes of a file and names the media extension they belong to.
/// TIFF is left out because camera RAW files share its header.
pub(super) fn media_extension_from_content(path: &Path) -> Option<&'static str> {
    let mut header = Vec::new();
    File::open(path)
        .ok()?
        .take(HEADER_LENGTH)
        .read_to_end(&mut header)
        .ok()?;
    media_extension_from_header(&header)
}

pub(super) fn media_extension_from_header(header: &[u8]) -> Option<&'static str> {
    if header.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("jpg");
    }
    if header.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("png");
    }
    if header.starts_with(b"RIFF") {
        return match header.get(8..12)? {
            b"WEBP" => Some("webp"),
            b"AVI " => Some("avi"),
            _ => None,
        };
    }
    if header.get(4..8) == Some(b"ftyp") {
        return iso_media_extension(header.get(8..12)?);
    }
    if matches!(header.get(4..8), Some(b"moov" | b"wide")) {
        return Some("mov");
    }
    transport_stream_extension(header)
}

fn iso_media_extension(brand: &[u8]) -> Option<&'static str> {
    let families = [
        (HEIC_BRANDS, "heic"),
        (HEIF_BRANDS, "heif"),
        (MOV_BRANDS, "mov"),
        (M4V_BRANDS, "m4v"),
        (MP4_BRANDS, "mp4"),
    ];
    families
        .into_iter()
        .find(|(brands, _)| brands.contains(&brand))
        .map(|(_, extension)| extension)
}

/// MPEG transport streams repeat the sync byte `0x47` at every packet start.
fn transport_stream_extension(header: &[u8]) -> Option<&'static str> {
    let has_sync_at = |offset: usize| header.get(offset) == Some(&0x47);
    if has_sync_at(0) && has_sync_at(TS_PACKET) {
        return Some("mts");
    }
    if has_sync_at(4) && has_sync_at(4 + M2TS_PACKET) {
        return Some("m2ts");
    }
    None
}
