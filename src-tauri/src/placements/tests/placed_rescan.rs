use crate::{
    models::{FileKind, FileRecord, MetadataResult, PlaceSuggestion, SuggestionKind},
    placements::continue_placed_suggestions,
    test_support::{
        media::{dated_metadata, file_record},
        month_library::{MonthLibrary, USA},
        placements::{detected, place_group},
    },
};

/// A USA trip a May rescan finds over the placed Yosemite files, plus `new` when given.
fn rescan(library: &MonthLibrary, new: Option<&str>) -> Vec<PlaceSuggestion> {
    let may = library.month("5");
    let yosemite = may.join(USA).join("Yosemite");
    let mut paths = vec![
        yosemite.join("05-03").join("a.jpg"),
        yosemite.join("05-04").join("b.jpg"),
    ];
    let mut dates = vec!["2026-05-03 10:00:00", "2026-05-04 10:00:00"];
    if let Some(date) = new {
        paths.push(may.join("20").join("new.jpg"));
        dates.push(date);
    }
    let files: Vec<FileRecord> = paths
        .iter()
        .map(|path| {
            let id = path.to_string_lossy().to_string();
            FileRecord {
                source_path: id.clone(),
                ..file_record(&id, "jpg", FileKind::Photo)
            }
        })
        .collect();
    let metadata: Vec<MetadataResult> = files
        .iter()
        .zip(&dates)
        .map(|(file, date)| dated_metadata(&file.id, date))
        .collect();
    let ids: Vec<&str> = files.iter().map(|file| file.id.as_str()).collect();
    let (first, last) = (dates[0], dates[dates.len() - 1]);
    let mut trip = detected(
        SuggestionKind::Trip,
        USA,
        vec![place_group("Yosemite", first, last, &ids)],
    );
    trip.country = Some(USA.to_owned());
    continue_placed_suggestions(
        &library.store,
        &library.settings,
        &may,
        &files,
        &metadata,
        vec![trip],
    )
    .expect("continued")
}

#[test]
fn a_rescan_of_placed_files_only_suggests_nothing() {
    let library = MonthLibrary::new();
    library.place_may_usa_trip();

    assert!(rescan(&library, None).is_empty());
}

#[test]
fn new_files_beside_placed_ones_continue_the_placed_folder() {
    let library = MonthLibrary::new();
    library.place_may_usa_trip();

    let found = rescan(&library, Some("2026-05-20 10:00:00"));

    let trip = &found[0];
    assert_eq!(trip.file_count, 1);
    assert_eq!(trip.places[0].file_ids.len(), 1);
    assert_eq!(
        (trip.start.as_str(), trip.places[0].start.as_str()),
        ("2026-05-20 10:00:00", "2026-05-20 10:00:00")
    );
    assert_eq!(trip.month_folder.as_deref(), Some("5"));
    assert!(trip
        .continues
        .as_ref()
        .is_some_and(|placed| placed.country_wide));
}
