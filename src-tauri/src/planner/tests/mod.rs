mod conflicts;
mod file_plan;
mod in_place;
mod place_fixtures;
mod place_folders;
mod place_validation;
mod streaming;

use std::path::Path;

use super::DestinationLayout;
use crate::{
    models::{FileKind, FileRecord},
    test_support::media::file_record,
};

fn dated(root: &Path) -> DestinationLayout {
    DestinationLayout::DatedFolders(root.to_path_buf())
}

fn photo_record(id: &str, extension: &str) -> FileRecord {
    file_record(id, extension, FileKind::Photo)
}
