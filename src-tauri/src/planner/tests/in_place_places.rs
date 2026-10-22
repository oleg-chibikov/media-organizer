use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{
    photo_record,
    place_fixtures::{group, suggestion},
};
use crate::{
    events,
    models::{PlaceSuggestion, SuggestionKind},
    planner::{stream_move_plan, DestinationLayout, PlaceFolders},
    settings::{set_rename_in_place, set_root_media_folder, SettingsStore},
    test_support::{events::RecordingSink, media::dated_metadata},
};

fn zoo_outing() -> PlaceSuggestion {
    let place = group("Zoo", "2025-03-01 10:00:00", "2025-03-01 11:00:00", &["a"]);
    suggestion(SuggestionKind::Outing, "Zoo", vec![place])
}

/// Plans `a` (in the outing) and `b` (in none), both inside `selected`.
fn planned_dirs(layout: &DestinationLayout, selected: &Path) -> Vec<PathBuf> {
    let mut records = vec![photo_record("a", "jpg"), photo_record("b", "jpg")];
    for record in &mut records {
        record.source_path = selected
            .join(&record.original_name)
            .to_string_lossy()
            .to_string();
    }
    let metadata = [
        dated_metadata("a", "2025-03-01 10:00:00"),
        dated_metadata("b", "2025-03-01 12:00:00"),
    ];
    let folders = PlaceFolders::new(&[zoo_outing()], &metadata).expect("valid suggestions");
    let sink = RecordingSink::default();
    stream_move_plan(&sink, layout, &records, &metadata, &folders);
    sink.payloads(events::PLAN_FILE)
        .iter()
        .map(|payload| PathBuf::from(payload["plan"]["destination_dir"].as_str().expect("a dir")))
        .collect()
}

#[test]
fn placed_files_leave_for_the_root_while_the_rest_rename_in_place() {
    let selected = PathBuf::from("Inbox");
    let layout = DestinationLayout::SameFolder {
        place_root: Some(PathBuf::from("Media")),
    };

    let dirs = planned_dirs(&layout, &selected);

    assert_eq!(
        dirs,
        [PathBuf::from("Media").join("2025").join("Zoo"), selected]
    );
}

#[test]
fn a_placed_file_without_a_resolved_root_is_an_error() {
    let layout = DestinationLayout::SameFolder { place_root: None };
    let sink = RecordingSink::default();
    let metadata = [dated_metadata("a", "2025-03-01 10:00:00")];
    let folders = PlaceFolders::new(&[zoo_outing()], &metadata).expect("valid suggestions");

    stream_move_plan(
        &sink,
        &layout,
        &[photo_record("a", "jpg")],
        &metadata,
        &folders,
    );

    assert_eq!(sink.payloads(events::PLAN_ERROR).len(), 1);
}

#[test]
fn the_layout_resolves_the_custom_root_for_places_when_renaming_in_place() {
    let temp = tempfile::tempdir().expect("temp dir");
    let (selected, root) = (temp.path().join("2025-03"), temp.path().join("Media"));
    fs::create_dir_all(&selected).expect("selected folder");
    fs::create_dir_all(&root).expect("root folder");
    let settings = SettingsStore::in_app_data(&temp.path().join("app-data"));
    let root_path = root.to_string_lossy().to_string();
    settings
        .update(|current| {
            set_rename_in_place(&set_root_media_folder(current, Some(root_path)), true)
        })
        .expect("settings");

    let plain = DestinationLayout::for_folder(&settings, &selected, false).expect("layout");
    let placing = DestinationLayout::for_folder(&settings, &selected, true).expect("layout");

    assert_eq!(plain.place_root(), None);
    assert_eq!(placing.place_root(), Some(root.as_path()));
}
