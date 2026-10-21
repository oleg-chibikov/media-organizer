use std::{collections::HashSet, path::Path};

use super::{dated, photo_record};
use crate::{
    models::{ConflictStatus, PlanStatus},
    planner::{destination_allocation::allocate_destination_path, file_plan::create_plan_for_file},
    test_support::media::dated_metadata,
};

const PLANNED_FILENAME: &str = "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro.mp4";

/// Writes a source file and an existing file at the planned destination.
fn existing_destination(root: &Path, source_bytes: &[u8], existing_bytes: &[u8]) -> String {
    let source = root.join("IMG_1001.mp4");
    std::fs::write(&source, source_bytes).expect("source file should be created");
    let destination_dir = root.join("2026").join("3").join("5");
    std::fs::create_dir_all(&destination_dir).expect("destination directory should be created");
    std::fs::write(destination_dir.join(PLANNED_FILENAME), existing_bytes)
        .expect("existing file should be created");
    source.to_string_lossy().to_string()
}

#[test]
fn applies_suffix_for_existing_destination_file() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let mut file = photo_record("IMG_1001", "mp4");
    file.source_path = existing_destination(temp_dir.path(), b"source-bytes", b"existing");
    let metadata = dated_metadata("IMG_1001", "2026-03-05 07:20:17");

    let plan = create_plan_for_file(
        &dated(temp_dir.path()),
        &file,
        Some(&metadata),
        &mut HashSet::new(),
    )
    .expect("plan should be created");

    assert_eq!(
        plan.destination_filename,
        "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro (1).mp4"
    );
}

#[test]
fn blocks_conflict_when_existing_destination_has_identical_content() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let mut file = photo_record("IMG_1001", "mp4");
    file.source_path = existing_destination(temp_dir.path(), b"same-bytes", b"same-bytes");
    let metadata = dated_metadata("IMG_1001", "2026-03-05 07:20:17");

    let plan = create_plan_for_file(
        &dated(temp_dir.path()),
        &file,
        Some(&metadata),
        &mut HashSet::new(),
    )
    .expect("plan should be created");

    assert!(matches!(plan.plan_status, PlanStatus::BlockedConflict));
    assert!(matches!(plan.conflict_status, ConflictStatus::Blocked));
    assert_eq!(plan.destination_filename, PLANNED_FILENAME);
}

#[test]
fn blocks_conflict_when_a_suffixed_destination_has_identical_content() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let mut file = photo_record("IMG_1001", "mp4");
    file.source_path = existing_destination(temp_dir.path(), b"copied-bytes", b"other");
    let suffixed = "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro (1).mp4";
    let destination_dir = temp_dir.path().join("2026").join("3").join("5");
    std::fs::write(destination_dir.join(suffixed), b"copied-bytes").expect("write copy");
    let metadata = dated_metadata("IMG_1001", "2026-03-05 07:20:17");

    let plan = create_plan_for_file(
        &dated(temp_dir.path()),
        &file,
        Some(&metadata),
        &mut HashSet::new(),
    )
    .expect("plan should be created");

    assert!(matches!(plan.plan_status, PlanStatus::BlockedConflict));
    assert_eq!(plan.destination_filename, suffixed);
}

#[test]
fn applies_suffix_for_duplicate_in_same_run() {
    let destination_dir = Path::new(r"C:\todo\Processed\2026\3\5");
    let stem_base = "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro";
    let mut reserved = HashSet::new();

    let source = Path::new(r"C:\todo\IMG_1001.mp4");
    let (_, filename_one, _, _) =
        allocate_destination_path(source, destination_dir, stem_base, "mp4", &mut reserved)
            .expect("first allocation should succeed");
    let (_, filename_two, _, _) =
        allocate_destination_path(source, destination_dir, stem_base, "mp4", &mut reserved)
            .expect("second allocation should succeed");

    assert_eq!(
        filename_one,
        "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro.mp4"
    );
    assert_eq!(
        filename_two,
        "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro (1).mp4"
    );
}
