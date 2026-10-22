use std::path::Path;

use super::{placed::PlacedPlace, PlacedFile, PlacedSuggestion};
use crate::models::Placement;

/// The record of `placement`'s folder, added empty when it is new.
pub(super) fn record_for<'a>(
    records: &'a mut Vec<PlacedSuggestion>,
    root: &Path,
    placement: &Placement,
) -> &'a mut PlacedSuggestion {
    let index = match records
        .iter()
        .position(|record| record.holds(root, placement))
    {
        Some(index) => index,
        None => {
            records.push(PlacedSuggestion {
                root: root.to_string_lossy().to_string(),
                kind: placement.kind,
                year: placement.year,
                title: placement.title.clone(),
                start: String::new(),
                end: String::new(),
                places: Vec::new(),
                root_files: Vec::new(),
                country: placement.country.clone(),
                month_folder: placement.month_folder.clone(),
            });
            records.len() - 1
        }
    };
    &mut records[index]
}

/// The files of the place folder `name`, added empty when it is new.
pub(super) fn place_files<'a>(
    record: &'a mut PlacedSuggestion,
    name: &str,
) -> &'a mut Vec<PlacedFile> {
    let index = match record.places.iter().position(|place| place.name == name) {
        Some(index) => index,
        None => {
            record.places.push(PlacedPlace {
                name: name.to_owned(),
                start: String::new(),
                end: String::new(),
                files: Vec::new(),
            });
            record.places.len() - 1
        }
    };
    &mut record.places[index].files
}
