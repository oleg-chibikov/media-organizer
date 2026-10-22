use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use chrono::NaiveDateTime;

use super::{
    destination_allocation::allocate_destination_path, place_folder::PlaceFolder, PlaceFolders,
};
use crate::{
    models::{MetadataResult, MetadataStatus, MovePlan, PlacedMove},
    path_safety::paths_equal_case_insensitive,
    placements::{PlacedFile, PlacedSuggestion},
    safe_fs::is_plain_file,
    scanner,
    trips::CHOSEN_DATE_FORMAT,
};

const PLACED_DATE_SOURCE: &str = "Placed folder record";

/// Plans the files earlier runs placed in a continued folder into its new day folders.
/// A file already in the right day folder, or no longer on disk, is left out.
pub(super) fn plan_placed_moves(
    root: &Path,
    folders: &PlaceFolders,
    records: &[PlacedSuggestion],
) -> Vec<PlacedMove> {
    let mut reserved_paths = HashSet::new();
    let mut moves = Vec::new();
    for folder in folders.continued() {
        let placement = &folder.placement;
        let Some(record) = records.iter().find(|record| record.holds(root, placement)) else {
            continue;
        };
        for file in record.files_in(placement.place.as_deref()) {
            moves.extend(plan_placed_file(root, folder, file, &mut reserved_paths));
        }
    }
    moves
}

fn plan_placed_file(
    root: &Path,
    folder: &PlaceFolder,
    file: &PlacedFile,
    reserved_paths: &mut HashSet<String>,
) -> Option<PlacedMove> {
    let date = NaiveDateTime::parse_from_str(&file.chosen_date, CHOSEN_DATE_FORMAT).ok()?;
    let source: PathBuf = root.join(&file.relative_path);
    let directory = folder.dir_under(root, &date);
    let in_place = source
        .parent()
        .is_some_and(|parent| paths_equal_case_insensitive(parent, &directory));
    if in_place || !is_plain_file(&source) {
        return None;
    }
    let stem = source.file_stem()?.to_string_lossy().to_string();
    let extension = source
        .extension()
        .map(|extension| extension.to_string_lossy().to_string())
        .unwrap_or_default();
    let (path, filename, conflict_status, plan_status) =
        allocate_destination_path(&source, &directory, &stem, &extension, reserved_paths).ok()?;
    let record = scanner::record_for_path(root, &source);
    Some(PlacedMove {
        metadata: placed_metadata(&record.id, &file.chosen_date),
        plan: MovePlan {
            file_id: record.id.clone(),
            original_name: record.original_name.clone(),
            destination_dir: directory.to_string_lossy().to_string(),
            destination_filename: filename,
            destination_path: path.to_string_lossy().to_string(),
            conflict_status,
            plan_status,
            placement: Some(folder.placement.clone()),
        },
        record,
    })
}

/// The date the registry kept, so the file isn't read again.
fn placed_metadata(file_id: &str, chosen_date: &str) -> MetadataResult {
    MetadataResult {
        file_id: file_id.to_owned(),
        chosen_date: Some(chosen_date.to_owned()),
        chosen_date_source: Some(PLACED_DATE_SOURCE.to_owned()),
        date_kind: None,
        raw_metadata_date: None,
        device_name: None,
        gps: None,
        metadata_status: MetadataStatus::Ready,
        error: None,
    }
}
