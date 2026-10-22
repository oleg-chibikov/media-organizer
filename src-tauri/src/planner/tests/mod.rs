mod conflicts;
mod file_plan;
mod home_city;
mod in_place;
mod in_place_places;
mod place_levels;
mod streaming;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::{plan_files, DestinationLayout, PlaceFolders, Reservations};
use crate::{
    models::{FileKind, FilePlace, FileRecord, HomeMatch},
    test_support::media::{dated_metadata, file_record},
};

fn dated(root: &Path) -> DestinationLayout {
    DestinationLayout::DatedFolders(root.to_path_buf())
}

fn photo_record(id: &str, extension: &str) -> FileRecord {
    file_record(id, extension, FileKind::Photo)
}

/// A place abroad in `country`, or in home's country with none.
fn place(name: &str, country: Option<&str>) -> FilePlace {
    FilePlace {
        place: name.to_owned(),
        country: country.unwrap_or("Australia").to_owned(),
        home: if country.is_some() {
            HomeMatch::Away
        } else {
            HomeMatch::Country
        },
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
    plan_files(
        layout,
        &records,
        &metadata,
        &folders,
        &mut Reservations::default(),
    )
    .plans
    .iter()
    .map(|plan| PathBuf::from(&plan.destination_dir))
    .collect()
}
