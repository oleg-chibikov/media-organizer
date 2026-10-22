use crate::{
    models::{ContinuedPlacement, LiftedMonth, PlaceSuggestion, Placement, SuggestionKind},
    test_support::{
        month_library::{MonthLibrary, USA},
        placements::{detected, moved_into, place_group, place_on_disk, placement},
    },
};

fn april_usa_trip() -> PlaceSuggestion {
    let yosemite = place_group(
        "yosemite",
        "2026-04-01 10:00:00",
        "2026-04-01 12:00:00",
        &["y1"],
    );
    let mut trip = detected(SuggestionKind::Trip, USA, vec![yosemite]);
    trip.country = Some(USA.to_owned());
    trip
}

fn continued(found: &PlaceSuggestion) -> &ContinuedPlacement {
    found.continues.as_ref().expect("continues")
}

#[test]
fn a_trip_abroad_in_a_fresh_month_stays_in_that_month() {
    let library = MonthLibrary::new();

    let found = library.detect(&library.month("04"), vec![april_usa_trip()]);

    assert_eq!(found[0].month_folder.as_deref(), Some("04"));
    assert_eq!(found[0].continues, None);
}

#[test]
fn an_outing_at_home_stays_in_the_month_scanned_from_a_day_folder() {
    let library = MonthLibrary::new();
    let zoo = place_group("Zoo", "2026-05-12 10:00:00", "2026-05-12 11:00:00", &["z"]);
    let day = library.month("5").join("12");

    let found = library.detect(
        &day,
        vec![detected(SuggestionKind::Outing, "Zoo", vec![zoo])],
    );

    assert_eq!(
        (found[0].year, found[0].month_folder.as_deref()),
        (2026, Some("5"))
    );
}

#[test]
fn a_country_placed_in_another_month_lifts_to_the_year() {
    let library = MonthLibrary::new();
    library.place_may_usa_trip();

    let found = library.detect(&library.month("4"), vec![april_usa_trip()]);

    assert_eq!(found[0].month_folder, None);
    assert_eq!(found[0].places[0].name, "Yosemite");
    let placed = continued(&found[0]);
    assert!(placed.country_wide);
    assert_eq!((placed.title.as_str(), placed.year), (USA, 2026));
    let lifted = LiftedMonth {
        month_folder: "5".to_owned(),
        file_count: 3,
    };
    assert_eq!(placed.lifted, [lifted]);
    assert_eq!(placed.places[0].start, "2026-05-03 10:00:00");
}

#[test]
fn a_country_in_the_same_month_continues_there() {
    let library = MonthLibrary::new();
    library.place_may_usa_trip();
    let mut trip = april_usa_trip();
    trip.places[0] = place_group(
        "Yosemite",
        "2026-05-20 10:00:00",
        "2026-05-20 10:00:00",
        &["y1"],
    );

    let found = library.detect(&library.month("5"), vec![trip]);

    assert_eq!(found[0].month_folder.as_deref(), Some("5"));
    let placed = continued(&found[0]);
    assert!(placed.country_wide && placed.lifted.is_empty());
}

#[test]
fn a_country_at_year_level_takes_new_months_without_a_gap_limit() {
    let library = MonthLibrary::new();
    let root = library.root();
    let year_level = Placement {
        country: Some(USA.to_owned()),
        ..placement(SuggestionKind::Trip, 2026, USA, Some("Boston"))
    };
    let path = "2026/USA/Boston/b.jpg";
    place_on_disk(
        &library.store,
        &root,
        &[moved_into(&root, &year_level, path, "2026-01-10 10:00:00")],
    );

    let found = library.detect(&library.month("4"), vec![april_usa_trip()]);

    assert_eq!(found[0].month_folder, None);
    assert!(continued(&found[0]).lifted.is_empty());
}

#[test]
fn a_scan_outside_month_folders_ignores_month_records() {
    let library = MonthLibrary::new();
    library.place_may_usa_trip();
    let mut trip = april_usa_trip();
    trip.places[0] = place_group(
        "Yosemite",
        "2026-05-06 10:00:00",
        "2026-05-06 10:00:00",
        &["y1"],
    );
    let camera = library.root().join("Camera");
    std::fs::create_dir_all(&camera).expect("camera folder");

    let found = library.detect(&camera, vec![trip]);

    assert_eq!(
        (found[0].month_folder.as_ref(), found[0].continues.as_ref()),
        (None, None)
    );
}
