use std::path::{Component, Path, PathBuf};

use super::{
    folders::place_folder,
    lookup::{place_files, record_for},
    recording::{forget_files, relative_to},
    spans::refresh_spans,
    PlacedFile, PlacedSuggestion,
};
use crate::models::Placement;

/// One file an undo moved from `destination` back to `source`.
pub(crate) struct UndoneMove {
    pub(crate) source: PathBuf,
    pub(crate) destination: PathBuf,
}

/// Drops the undone files from the records under `root`. A placed file the run moved to
/// a new day folder, or up from a month folder to the year, goes back to the record of the
/// place folder it returned to.
pub(crate) fn restore_files(
    records: &mut Vec<PlacedSuggestion>,
    root: &Path,
    undone: &[UndoneMove],
) {
    let returned: Vec<(Placement, PlacedFile)> = undone
        .iter()
        .filter_map(|undone| returned_file(records, root, undone))
        .collect();
    let destinations: Vec<&Path> = undone
        .iter()
        .map(|undone| undone.destination.as_path())
        .collect();
    forget_files(records, root, &destinations);
    for (placement, file) in returned {
        let record = record_for(records, root, &placement);
        match &placement.place {
            Some(name) => place_files(record, name).push(file),
            None => record.root_files.push(file),
        }
    }
    records.iter_mut().for_each(refresh_spans);
}

/// The placement and entry of a recorded file whose source sits in a place folder of the
/// same record, at its level or in the month folder it was lifted from.
fn returned_file(
    records: &[PlacedSuggestion],
    root: &Path,
    undone: &UndoneMove,
) -> Option<(Placement, PlacedFile)> {
    let relative = relative_to(root, &undone.destination)?;
    let (record, place, file) = records
        .iter()
        .filter(|record| record.is_in(root))
        .find_map(|record| {
            let places = record
                .places
                .iter()
                .map(|place| (Some(place.name.as_str()), &place.files));
            places
                .chain([(None, &record.root_files)])
                .find_map(|(place, files)| {
                    let file = files.iter().find(|file| file.relative_path == relative)?;
                    Some((record, place, file))
                })
        })?;
    let placement = record.placement(place);
    let lifted_from = Placement {
        month_folder: month_of(root, record.year, &undone.source),
        ..placement.clone()
    };
    let candidates = [placement.clone()]
        .into_iter()
        .chain(placement.month_folder.is_none().then_some(lifted_from));
    let returned = candidates
        .into_iter()
        .find(|candidate| undone.source.starts_with(place_folder(root, candidate)))?;
    Some((
        returned,
        PlacedFile {
            relative_path: relative_to(root, &undone.source)?,
            chosen_date: file.chosen_date.clone(),
        },
    ))
}

/// The folder right below `<root>/<year>` that `path` sits in.
fn month_of(root: &Path, year: i32, path: &Path) -> Option<String> {
    match path
        .strip_prefix(root.join(year.to_string()))
        .ok()?
        .components()
        .next()?
    {
        Component::Normal(name) => Some(name.to_string_lossy().to_string()),
        _ => None,
    }
}
