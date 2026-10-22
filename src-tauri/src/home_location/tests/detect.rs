use super::fixtures::{daily, empty_store, saved_home, store_with_home, HOUSTON, LIVERPOOL_NSW};
use crate::{
    home_location::detect_place_suggestions,
    models::{HomeSource, SuggestionKind},
};

#[test]
fn a_saved_home_finds_the_trip_in_one_month_folder() {
    let (_temp, store) = store_with_home("Sydney", (-33.8679, 151.2073), HomeSource::Manual);
    let (files, metadata) = daily("2025-10-01", 20, HOUSTON);

    let found = detect_place_suggestions(&store, &files, &metadata)
        .expect("suggestions")
        .suggestions;

    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].kind, SuggestionKind::Trip);
    assert_eq!(found[0].title, "United States");
    assert_eq!(
        saved_home(&store).map(|home| home.name),
        Some("Sydney".to_owned())
    );
}

#[test]
fn a_scan_with_60_days_at_home_saves_it_as_learned() {
    let (_temp, store) = empty_store();
    let (files, metadata) = daily("2025-01-01", 60, LIVERPOOL_NSW);

    detect_place_suggestions(&store, &files, &metadata).expect("suggestions");

    let home = saved_home(&store).expect("a learned home");
    assert_eq!(home.name, "Liverpool");
    assert_eq!(home.country, "Australia");
    assert_eq!(home.source, HomeSource::Learned);
}

#[test]
fn a_smaller_scan_infers_home_for_itself_and_saves_none() {
    let (_temp, store) = empty_store();
    let (mut files, mut metadata) = daily("2025-01-01", 59, LIVERPOOL_NSW);
    let (trip_files, trip_metadata) = daily("2025-03-10", 10, HOUSTON);
    files.extend(trip_files);
    metadata.extend(trip_metadata);

    let found = detect_place_suggestions(&store, &files, &metadata)
        .expect("suggestions")
        .suggestions;

    let trips: Vec<&str> = found
        .iter()
        .filter(|suggestion| suggestion.kind == SuggestionKind::Trip)
        .map(|suggestion| suggestion.title.as_str())
        .collect();
    assert_eq!(trips, ["United States"], "{found:?}");
    assert_eq!(saved_home(&store), None);
}

#[test]
fn a_large_scan_keeps_the_saved_home() {
    let (_temp, store) = store_with_home("Houston", HOUSTON, HomeSource::Manual);
    let (files, metadata) = daily("2025-01-01", 90, LIVERPOOL_NSW);

    detect_place_suggestions(&store, &files, &metadata).expect("suggestions");

    let home = saved_home(&store).expect("the saved home");
    assert_eq!(
        (home.name.as_str(), home.source),
        ("Houston", HomeSource::Manual)
    );
}
