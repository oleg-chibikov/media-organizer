use std::path::Path;

use super::dated;
use crate::{
    models::FileKind,
    planner::{plan_files, PlaceFolders, Reservations},
    test_support::media::{dated_metadata, file_record},
};

#[test]
fn plans_only_photos_and_videos() {
    let photo = file_record("IMG_1001", "jpg", FileKind::Photo);
    let notes = file_record("notes", "txt", FileKind::Unsupported);
    let metadata = dated_metadata("IMG_1001", "2026-03-05 07:20:17");

    let planned = plan_files(
        &dated(Path::new("MediaRoot")),
        &[photo, notes],
        &[metadata],
        &PlaceFolders::default(),
        &mut Reservations::default(),
    );

    assert_eq!(planned.plans.len(), 1);
    assert_eq!(planned.plans[0].file_id, "IMG_1001");
    assert!(planned.errors.is_empty());
}
