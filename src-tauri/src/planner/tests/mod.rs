mod conflicts;
mod continued;
mod country_folders;
mod file_plan;
mod in_place;
mod in_place_places;
mod month_places;
mod place_fixtures;
mod place_folders;
mod place_validation;
mod placed_october;
mod placed_rescan;
mod streaming;

use std::path::{Path, PathBuf};

use super::{stream_move_plan, DestinationLayout, PlaceFolders};
use crate::{
    events,
    models::{FileKind, FileRecord, PlaceSuggestion},
    test_support::{
        events::RecordingSink,
        media::{dated_metadata, file_record},
    },
};

fn dated(root: &Path) -> DestinationLayout {
    DestinationLayout::DatedFolders(root.to_path_buf())
}

fn photo_record(id: &str, extension: &str) -> FileRecord {
    file_record(id, extension, FileKind::Photo)
}

/// The folder a photo taken on `date` gets under `root` when it joins `trip`.
fn planned_dir(root: &Path, trip: &PlaceSuggestion, id: &str, date: &str) -> PathBuf {
    let metadata = [dated_metadata(id, date)];
    let folders = PlaceFolders::new(std::slice::from_ref(trip), &metadata).expect("folders");
    let sink = RecordingSink::default();
    let files = [photo_record(id, "jpg")];
    stream_move_plan(&sink, 0, &dated(root), &files, &metadata, &folders);
    PathBuf::from(
        sink.payloads(events::PLAN_FILE)[0]["plan"]["destination_dir"]
            .as_str()
            .expect("a dir"),
    )
}
