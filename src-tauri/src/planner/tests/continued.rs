use std::{fs, path::PathBuf};

use super::placed_october::{january_trip, planned_dir, PlacedOctober, USA};
use crate::models::PlanStatus;

#[test]
fn a_continued_place_moves_its_single_day_files_into_day_folders() {
    let october = PlacedOctober::new();
    let trip = october.november_trip();
    let vancouver = october.root().join("2025").join(USA).join("Vancouver");

    assert_eq!(
        planned_dir(&october.root(), &trip, "v1", "2025-11-01 09:00:00"),
        vancouver.join("11-01")
    );
    let moves = october.placed_moves(&trip);

    assert_eq!(moves.len(), 1, "{moves:?}");
    let plan = &moves[0].plan;
    assert_eq!(
        PathBuf::from(&plan.destination_dir),
        vancouver.join("10-31")
    );
    assert_eq!(plan.destination_filename, "c.jpg");
    assert!(matches!(plan.plan_status, PlanStatus::Ready));
    assert_eq!(
        moves[0].record.source_path,
        vancouver.join("c.jpg").to_string_lossy()
    );
    assert_eq!(
        moves[0].metadata.chosen_date.as_deref(),
        Some("2025-10-31 18:00:00")
    );
    let placement = plan.placement.as_ref().expect("a placement");
    assert_eq!(
        (placement.title.as_str(), placement.place.as_deref()),
        (USA, Some("Vancouver"))
    );
}

#[test]
fn a_placed_file_gone_from_disk_is_left_out() {
    let october = PlacedOctober::new();
    fs::remove_file(
        october
            .root()
            .join("2025")
            .join(USA)
            .join("Vancouver")
            .join("c.jpg"),
    )
    .expect("delete");

    assert!(october.placed_moves(&october.november_trip()).is_empty());
}

#[test]
fn a_renamed_continued_trip_is_a_new_trip() {
    let october = PlacedOctober::new();
    let mut trip = january_trip(&october, "Vancouver");
    trip.title = "Canada".to_owned();

    let dir = planned_dir(&october.root(), &trip, "v1", "2026-01-02 09:00:00");

    assert_eq!(
        dir,
        october.root().join("2026").join("Canada").join("Vancouver")
    );
    assert!(october.placed_moves(&trip).is_empty());
}

#[test]
fn a_new_year_part_goes_under_the_placed_year() {
    let october = PlacedOctober::new();
    let trip = january_trip(&october, "Whistler");

    let dir = planned_dir(&october.root(), &trip, "v1", "2026-01-02 09:00:00");

    assert_eq!(dir, october.root().join("2025").join(USA).join("Whistler"));
}
