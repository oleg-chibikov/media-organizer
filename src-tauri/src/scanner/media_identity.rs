use std::path::Path;

use super::{classification::classify_extension, file_signature::media_extension_from_content};
use crate::{filename, models::FileKind};

const MAX_PLAIN_EXTENSION_LENGTH: usize = 5;

/// What a file is and which name parts to keep when it moves.
pub(super) struct MediaIdentity {
    pub kind: FileKind,
    pub extension: String,
    pub original_name: String,
}

impl MediaIdentity {
    pub(super) fn from_name(path: &Path) -> Self {
        let extension = file_extension(path);
        Self {
            kind: classify_extension(extension),
            extension: extension.to_string(),
            original_name: filename::parse_original_name(file_name(path)),
        }
    }

    /// Reads the file header when the name has no usable extension, as in `name(IMG_1.jpg)`.
    pub(super) fn from_name_or_content(path: &Path) -> Self {
        let by_name = Self::from_name(path);
        if !matches!(by_name.kind, FileKind::Unknown) || is_plain_extension(&by_name.extension) {
            return by_name;
        }
        let Some(extension) = media_extension_from_content(path) else {
            return by_name;
        };
        let original_name = filename::original_name_from_stem(file_name(path));
        Self {
            kind: classify_extension(extension),
            extension: extension.to_string(),
            original_name: strip_media_extension(&original_name).to_string(),
        }
    }
}

/// A real-looking extension is trusted, so sidecars such as `.thm` (a JPEG inside) stay skipped.
fn is_plain_extension(extension: &str) -> bool {
    (1..=MAX_PLAIN_EXTENSION_LENGTH).contains(&extension.len())
        && extension.bytes().all(|byte| byte.is_ascii_alphanumeric())
        && extension.bytes().any(|byte| byte.is_ascii_alphabetic())
}

fn strip_media_extension(name: &str) -> &str {
    match name.rsplit_once('.') {
        Some((stem, extension))
            if !stem.is_empty()
                && matches!(
                    classify_extension(extension),
                    FileKind::Photo | FileKind::Video
                ) =>
        {
            stem
        }
        _ => name,
    }
}

fn file_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
}

fn file_extension(path: &Path) -> &str {
    path.extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
}
