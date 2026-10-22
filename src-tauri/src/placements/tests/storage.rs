use std::fs;

use tempfile::tempdir;

use crate::{
    models::SuggestionKind,
    placements::PlacementStore,
    test_support::placements::{moved_into, place_on_disk, placement},
};

#[test]
fn app_data_without_a_registry_loads_empty() {
    let temp = tempdir().expect("temp dir");
    fs::create_dir_all(temp.path().join("settings")).expect("settings dir");

    let store = PlacementStore::in_app_data(temp.path());

    assert_eq!(store.load().expect("load"), []);
    assert_eq!(store.load_existing().expect("load existing"), []);
}

#[test]
fn a_record_without_places_or_root_files_loads() {
    let temp = tempdir().expect("temp dir");
    let dir = temp.path().join("placements");
    fs::create_dir_all(&dir).expect("placements dir");
    let record = r#"{"version": 1, "suggestions": [{"root": "/r", "kind": "trip",
        "year": 2025, "title": "Japan", "start": "2025-04-01 10:00:00",
        "end": "2025-04-02 10:00:00"}]}"#;
    fs::write(dir.join("placed-suggestions.json"), record).expect("write registry");

    let loaded = PlacementStore::in_app_data(temp.path())
        .load()
        .expect("load");

    assert_eq!(loaded.len(), 1);
    assert!(loaded[0].places.is_empty() && loaded[0].root_files.is_empty());
}

#[test]
fn a_record_whose_folder_was_deleted_is_dropped() {
    let temp = tempdir().expect("temp dir");
    let (store, root) = (
        PlacementStore::in_app_data(&temp.path().join("app-data")),
        temp.path().join("Root"),
    );
    let japan = placement(SuggestionKind::Trip, 2025, "Japan", Some("Tokyo"));
    let peru = placement(SuggestionKind::Trip, 2025, "Peru", Some("Lima"));
    place_on_disk(
        &store,
        &root,
        &[
            moved_into(
                &root,
                &japan,
                "2025/Japan/Tokyo/a.jpg",
                "2025-04-01 10:00:00",
            ),
            moved_into(&root, &peru, "2025/Peru/Lima/b.jpg", "2025-08-01 10:00:00"),
        ],
    );
    fs::remove_dir_all(root.join("2025").join("Peru")).expect("delete Peru");

    let titles: Vec<String> = store
        .load_existing()
        .expect("load existing")
        .into_iter()
        .map(|record| record.title)
        .collect();

    assert_eq!(titles, ["Japan"]);
    assert_eq!(store.load().expect("saved").len(), 1);
}
