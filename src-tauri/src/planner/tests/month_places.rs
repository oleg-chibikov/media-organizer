use std::path::PathBuf;

use super::planned_dir;
use crate::{
    models::{PlaceSuggestion, PlacedMove, SuggestionKind},
    planner::plan_continued_moves,
    test_support::{
        media::dated_metadata,
        month_library::{MonthLibrary, USA},
        placements::{detected, place_group},
    },
};

/// An April USA trip: a Yosemite day, a Boston day and a root file.
fn april_trip() -> PlaceSuggestion {
    let mut trip = detected(
        SuggestionKind::Trip,
        USA,
        vec![
            place_group(
                "Yosemite",
                "2026-04-01 10:00:00",
                "2026-04-01 10:00:00",
                &["y1"],
            ),
            place_group(
                "Boston",
                "2026-04-02 10:00:00",
                "2026-04-02 10:00:00",
                &["b1"],
            ),
        ],
    );
    trip.root_file_ids = vec!["r1".to_owned()];
    trip.country = Some(USA.to_owned());
    trip
}

fn placed_moves(library: &MonthLibrary, trip: &PlaceSuggestion) -> Vec<(String, PathBuf)> {
    let metadata = [dated_metadata("y1", "2026-04-01 10:00:00")];
    let moves: Vec<PlacedMove> = plan_continued_moves(
        &library.settings,
        &library.store,
        &library.month("4"),
        std::slice::from_ref(trip),
        &metadata,
    )
    .expect("planned");
    moves
        .into_iter()
        .map(|placed| {
            (
                placed.plan.destination_filename,
                PathBuf::from(placed.plan.destination_dir),
            )
        })
        .collect()
}

#[test]
fn a_fresh_month_files_places_inside_the_month() {
    let library = MonthLibrary::new();
    let trip = library
        .detect(&library.month("4"), vec![april_trip()])
        .remove(0);
    let usa = library.root().join("2026").join("4").join(USA);

    let dir = planned_dir(&library.root(), &trip, "b1", "2026-04-02 10:00:00");

    assert_eq!(dir, usa.join("Boston"));
    assert!(placed_moves(&library, &trip).is_empty());
}

#[test]
fn an_outing_at_home_goes_to_its_month_folder() {
    let library = MonthLibrary::new();
    let zoo = place_group("Zoo", "2026-04-12 10:00:00", "2026-04-12 11:00:00", &["z"]);
    let outing = detected(SuggestionKind::Outing, "Zoo", vec![zoo]);
    let outing = library.detect(&library.month("04"), vec![outing]).remove(0);

    let dir = planned_dir(&library.root(), &outing, "z", "2026-04-12 10:00:00");

    assert_eq!(dir, library.root().join("2026").join("04").join("Zoo"));
}

#[test]
fn a_lifted_country_plans_new_and_placed_files_at_year_level() {
    let library = MonthLibrary::new();
    library.place_may_usa_trip();
    let trip = library
        .detect(&library.month("4"), vec![april_trip()])
        .remove(0);
    let usa = library.root().join("2026").join(USA);

    // Yosemite spans Apr 1 to May 4, over 31 days, so it gets month and day folders.
    let dir = planned_dir(&library.root(), &trip, "y1", "2026-04-01 10:00:00");
    assert_eq!(dir, usa.join("Yosemite").join("4").join("1"));
    assert_eq!(
        planned_dir(&library.root(), &trip, "b1", "2026-04-02 10:00:00"),
        usa.join("Boston")
    );

    let mut moves = placed_moves(&library, &trip);
    moves.sort();
    assert_eq!(
        moves,
        [
            ("a.jpg".to_owned(), usa.join("Yosemite").join("5").join("3")),
            ("b.jpg".to_owned(), usa.join("Yosemite").join("5").join("4")),
            ("r.jpg".to_owned(), usa.join("5").join("5")),
        ]
    );
}

#[test]
fn a_lifted_place_the_new_scan_lacks_keeps_its_own_span() {
    let library = MonthLibrary::new();
    library.place_may_usa_trip();
    let mut trip = april_trip();
    trip.places.remove(0);
    let trip = library.detect(&library.month("4"), vec![trip]).remove(0);
    let yosemite = library.root().join("2026").join(USA).join("Yosemite");

    let mut moves = placed_moves(&library, &trip);
    moves.sort();

    assert_eq!(moves[0], ("a.jpg".to_owned(), yosemite.join("05-03")));
    assert_eq!(moves[1], ("b.jpg".to_owned(), yosemite.join("05-04")));
}
