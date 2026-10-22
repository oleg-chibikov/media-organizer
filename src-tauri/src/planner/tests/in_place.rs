use std::{collections::HashSet, fs, path::Path};

use super::photo_record;
use crate::planner::PlaceFolders;
use crate::{
    models::{ConflictStatus, FileRecord, PlanStatus},
    planner::{file_plan::create_plan_for_file, DestinationLayout},
    test_support::media::dated_metadata,
};

const SAME_FOLDER: DestinationLayout = DestinationLayout::SameFolder { place_root: None };
const PLANNED_FILENAME: &str = "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro.mp4";

fn record_at(folder: &Path, file_name: &str, bytes: &[u8]) -> FileRecord {
    fs::create_dir_all(folder).expect("folder should be created");
    let source = folder.join(file_name);
    fs::write(&source, bytes).expect("source file should be created");
    let mut file = photo_record("IMG_1001", "mp4");
    file.source_path = source.to_string_lossy().to_string();
    file
}

#[test]
fn renames_into_the_folder_the_file_is_in() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let folder = temp_dir.path().join("trip");
    let file = record_at(&folder, "IMG_1001.mp4", b"video");
    let metadata = dated_metadata("IMG_1001", "2026-03-05 07:20:17");

    let plan = create_plan_for_file(
        &SAME_FOLDER,
        &file,
        Some(&metadata),
        &PlaceFolders::default(),
        &mut HashSet::new(),
    )
    .expect("plan should be created");

    assert!(matches!(plan.plan_status, PlanStatus::Ready));
    assert_eq!(Path::new(&plan.destination_dir), folder);
    assert_eq!(plan.destination_filename, PLANNED_FILENAME);
}

#[test]
fn keeps_a_file_that_already_has_its_planned_name() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let file = record_at(temp_dir.path(), PLANNED_FILENAME, b"video");
    let metadata = dated_metadata("IMG_1001", "2026-03-05 07:20:17");

    let plan = create_plan_for_file(
        &SAME_FOLDER,
        &file,
        Some(&metadata),
        &PlaceFolders::default(),
        &mut HashSet::new(),
    )
    .expect("plan should be created");

    assert!(matches!(plan.plan_status, PlanStatus::AlreadyNamed));
    assert!(matches!(plan.conflict_status, ConflictStatus::None));
    assert_eq!(plan.destination_path, file.source_path);
}

#[test]
fn keeps_a_file_that_already_has_a_numbered_name() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    fs::write(temp_dir.path().join(PLANNED_FILENAME), b"other video")
        .expect("taken name should be created");
    let numbered = "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro (1).mp4";
    let file = record_at(temp_dir.path(), numbered, b"video");
    let metadata = dated_metadata("IMG_1001", "2026-03-05 07:20:17");

    let plan = create_plan_for_file(
        &SAME_FOLDER,
        &file,
        Some(&metadata),
        &PlaceFolders::default(),
        &mut HashSet::new(),
    )
    .expect("plan should be created");

    assert!(matches!(plan.plan_status, PlanStatus::AlreadyNamed));
    assert_eq!(plan.destination_filename, numbered);
}
