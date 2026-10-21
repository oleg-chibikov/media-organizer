mod conflicts;
mod file_plan;
mod streaming;

use crate::{
    models::{FileKind, FileRecord},
    test_support::media::file_record,
};

fn photo_record(id: &str, extension: &str) -> FileRecord {
    file_record(id, extension, FileKind::Photo)
}
