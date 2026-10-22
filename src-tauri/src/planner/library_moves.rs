use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

use super::{
    destination_allocation::allocate_destination_path, nearby_folders::NearbyFolder,
    place_folders::PlaceFolders, place_level::dated_dir, Reservations,
};
use crate::{
    models::{FilePlace, FileRecord, LibraryFolderMove, MovePlan, PlanStatus, ScanStatus},
    path_safety::path_is_within_case_insensitive,
    scanner::record_for_path,
};

/// Files of earlier runs that move into their place's new folder, keeping their names.
#[derive(Debug, Default)]
pub(crate) struct LibraryMoves {
    pub records: Vec<FileRecord>,
    pub plans: Vec<MovePlan>,
    pub folders: Vec<LibraryFolderMove>,
}

impl PlaceFolders {
    /// Plans each photo and video of the `nearby` folders into the folder of its day. Files
    /// under `selected` belong to the scan and are planned with it.
    pub(crate) fn library_moves(
        &self,
        root: &Path,
        nearby: &[NearbyFolder],
        selected: &Path,
        reservations: &mut Reservations,
    ) -> LibraryMoves {
        let mut moves = LibraryMoves::default();
        for folder in nearby {
            let Some(place) = self.key_place(&folder.key) else {
                continue;
            };
            let target = self.key_folders(&folder.key);
            let planned_before = moves.plans.len();
            for (date, day_dir) in &folder.days {
                let day_target = dated_dir(*date, &target, false);
                if day_target != *day_dir {
                    let from = root.join(day_dir);
                    let to = root.join(day_target);
                    let reserved = reservations.paths();
                    plan_folder(&mut moves, root, &from, &to, place, selected, reserved);
                }
            }
            for plan in &moves.plans[planned_before..] {
                reservations.hold(plan);
            }
            let files = (moves.plans.len() - planned_before) as u64;
            if let (true, Some((first, _))) = (files > 0, folder.days.first()) {
                moves.folders.push(LibraryFolderMove {
                    from: path_text(&root.join(&folder.dir)),
                    to: path_text(&root.join(dated_dir(*first, &target, true))),
                    files,
                });
            }
        }
        moves
    }

    fn key_place(&self, key: &super::place_folders::PlaceKey) -> Option<&FilePlace> {
        self.by_file
            .values()
            .find(|(_, file_key)| file_key == key)
            .map(|(place, _)| place)
    }
}

fn plan_folder(
    moves: &mut LibraryMoves,
    root: &Path,
    from: &Path,
    to: &Path,
    place: &FilePlace,
    selected: &Path,
    reserved: &mut HashSet<String>,
) {
    let files = WalkDir::new(from)
        .into_iter()
        .flatten()
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| !path_is_within_case_insensitive(path, selected));
    for source in files {
        let record = record_for_path(root, &source);
        if !matches!(record.scan_status, ScanStatus::Discovered) {
            continue;
        }
        let Some(plan) = plan_file(&record, &source, from, to, place, reserved) else {
            continue;
        };
        moves.records.push(record);
        moves.plans.push(plan);
    }
}

/// The same name and subfolder under `to`, with a suffix when the name is taken.
fn plan_file(
    record: &FileRecord,
    source: &Path,
    from: &Path,
    to: &Path,
    place: &FilePlace,
    reserved: &mut HashSet<String>,
) -> Option<MovePlan> {
    let relative = source.strip_prefix(from).ok()?;
    let dir: PathBuf = to.join(relative.parent()?);
    let stem = source.file_stem()?.to_string_lossy();
    let extension = source
        .extension()
        .map(|ext| ext.to_string_lossy())
        .unwrap_or_default();
    let (path, filename, conflict_status, plan_status) =
        allocate_destination_path(source, &dir, &stem, &extension, reserved).ok()?;
    // A copy of the same bytes already at the destination keeps the file where it is.
    if !matches!(plan_status, PlanStatus::Ready) {
        return None;
    }
    Some(MovePlan {
        file_id: record.id.clone(),
        original_name: record.original_name.clone(),
        destination_dir: path_text(&dir),
        destination_filename: filename,
        destination_path: path_text(&path),
        conflict_status,
        plan_status,
        place: Some(place.clone()),
    })
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
