mod lookups;
mod planning;

use std::{path::Path, sync::Arc};

use super::{CurrentScan, PlanContext, ScanSession};
use crate::{
    models::{FileKind, FileRecord},
    placements::PlacementStore,
    settings::SettingsStore,
    test_support::media::file_record,
};

/// A session for `folder`, with settings and placements kept in `app_data`.
fn session_in(folder: &Path, app_data: &Path) -> Arc<ScanSession> {
    let context = PlanContext {
        folder: folder.to_path_buf(),
        settings: SettingsStore::in_app_data(app_data),
        placements: PlacementStore::in_app_data(app_data),
    };
    CurrentScan::default().begin(4, context)
}

/// A photo named `name` inside `folder/sub`, so two folders can hold the same name.
fn photo_in(folder: &Path, sub: &str, name: &str) -> FileRecord {
    let mut record = file_record(&format!("{sub}/{name}"), "jpg", FileKind::Photo);
    record.original_name = name.to_string();
    let source = folder.join(sub).join(format!("{name}.jpg"));
    std::fs::create_dir_all(source.parent().expect("parent")).expect("create folder");
    std::fs::write(&source, sub.as_bytes()).expect("write photo");
    record.source_path = source.to_string_lossy().to_string();
    record
}
