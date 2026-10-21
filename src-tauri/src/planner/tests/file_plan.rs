use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use super::photo_record;
use crate::{
    models::PlanStatus, planner::file_plan::create_plan_for_file,
    test_support::media::dated_metadata,
};

#[test]
fn plans_forced_legacy_processed_root_destination_and_filename() {
    let mut reserved = HashSet::new();
    let file = photo_record("IMG_1001", "mp4");
    let metadata = dated_metadata("IMG_1001", "2026-03-05 07:20:17");

    let root = Path::new("todo").join("Processed");
    let plan = create_plan_for_file(&root, &file, Some(&metadata), &mut reserved)
        .expect("plan should be created");

    assert_eq!(
        PathBuf::from(&plan.destination_dir),
        root.join("2026").join("3").join("5")
    );
    assert_eq!(
        plan.destination_filename,
        "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro.mp4"
    );
}

#[test]
fn plans_custom_root_destination_and_filename() {
    let mut reserved = HashSet::new();
    let file = photo_record("IMG_1001", "mp4");
    let metadata = dated_metadata("IMG_1001", "2026-03-05 07:20:17");

    let root = Path::new("MediaRoot");
    let plan = create_plan_for_file(root, &file, Some(&metadata), &mut reserved)
        .expect("plan should be created");

    assert_eq!(
        PathBuf::from(&plan.destination_dir),
        root.join("2026").join("3").join("5")
    );
    assert_eq!(
        plan.destination_filename,
        "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro.mp4"
    );
}

#[test]
fn blocks_plan_when_metadata_not_ready() {
    let mut reserved = HashSet::new();
    let file = photo_record("IMG_1001", "jpg");

    let plan = create_plan_for_file(Path::new(r"C:\todo"), &file, None, &mut reserved).unwrap();

    assert!(matches!(plan.plan_status, PlanStatus::BlockedNoDate));
    assert!(plan.destination_path.is_empty());
}

#[test]
fn dry_run_planning_does_not_create_destination_directories() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let mut reserved = HashSet::new();
    let file = photo_record("IMG_1001", "jpg");
    let metadata = dated_metadata("IMG_1001", "2026-03-05 07:20:17");

    let plan = create_plan_for_file(temp_dir.path(), &file, Some(&metadata), &mut reserved)
        .expect("plan should be created");

    assert!(matches!(plan.plan_status, PlanStatus::Ready));
    let destination_dir = temp_dir.path().join("2026").join("3").join("5");
    assert!(
        !destination_dir.exists(),
        "planner dry-run must not create filesystem directories"
    );
}
