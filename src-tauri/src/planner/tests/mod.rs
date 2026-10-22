mod conflicts;
mod file_plan;
mod in_place;
mod in_place_places;
mod place_levels;
mod streaming;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::{stream_move_plan, DestinationLayout, PlaceFolders, Reservations};
use crate::{
    events,
    models::{FileKind, FilePlace, FileRecord},
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

fn place(name: &str, country: Option<&str>) -> FilePlace {
    FilePlace {
        place: name.to_owned(),
        country: country.map(str::to_owned),
    }
}

/// Plans `(file id, chosen date, place)` files into `layout` and returns each folder.
fn planned_dirs(
    layout: &DestinationLayout,
    files: &[(&str, &str, Option<FilePlace>)],
) -> Vec<PathBuf> {
    let records: Vec<_> = files
        .iter()
        .map(|(id, ..)| photo_record(id, "jpg"))
        .collect();
    let metadata: Vec<_> = files
        .iter()
        .map(|(id, date, _)| dated_metadata(id, date))
        .collect();
    let places: HashMap<String, FilePlace> = files
        .iter()
        .filter_map(|(id, _, place)| Some(((*id).to_owned(), place.clone()?)))
        .collect();
    let folders = PlaceFolders::new(&places, &metadata).expect("valid places");
    let sink = RecordingSink::default();
    stream_move_plan(
        &sink,
        0,
        layout,
        &records,
        &metadata,
        &folders,
        &mut Reservations::default(),
    );
    sink.payloads(events::PLAN_FILE)
        .iter()
        .map(|payload| PathBuf::from(payload["plan"]["destination_dir"].as_str().expect("a dir")))
        .collect()
}
