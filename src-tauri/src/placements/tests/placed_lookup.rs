use std::path::{Path, PathBuf};

use crate::{
    models::SuggestionKind,
    placements::{placed::PlacedPlace, placed_files, PlacedFile, PlacedSuggestion},
};

fn trip(root: &Path, files: &[&str]) -> PlacedSuggestion {
    let placed = |path: &&str| PlacedFile {
        relative_path: (*path).to_owned(),
        chosen_date: "2026-05-03 10:00:00".to_owned(),
    };
    PlacedSuggestion {
        root: root.to_string_lossy().to_string(),
        kind: SuggestionKind::Trip,
        year: 2026,
        title: "USA".to_owned(),
        start: "2026-05-01 00:00:00".to_owned(),
        end: "2026-05-05 00:00:00".to_owned(),
        places: vec![PlacedPlace {
            name: "Yosemite".to_owned(),
            start: "2026-05-01 00:00:00".to_owned(),
            end: "2026-05-05 00:00:00".to_owned(),
            files: files.iter().map(placed).collect(),
        }],
        root_files: Vec::new(),
        country: None,
        month_folder: None,
    }
}

#[test]
fn only_the_asked_files_are_looked_up() {
    let root = PathBuf::from("/Library");
    let records = [trip(
        &root,
        &["2026/USA/Yosemite/a.jpg", "2026/USA/Yosemite/b.jpg"],
    )];
    let asked = root.join("2026/USA/Yosemite/a.jpg");
    let outside = PathBuf::from("/Elsewhere/2026/USA/Yosemite/b.jpg");

    let placed = placed_files(&records, &root, [asked.as_path(), outside.as_path()]);

    assert_eq!(placed.keys().collect::<Vec<_>>(), [&asked]);
    assert_eq!(placed[&asked].place.as_deref(), Some("Yosemite"));
}

#[test]
fn no_file_under_the_root_finds_nothing() {
    let root = PathBuf::from("/Library");
    let records = [trip(&root, &["2026/USA/Yosemite/a.jpg"])];
    let outside = PathBuf::from("/Elsewhere/a.jpg");

    assert!(placed_files(&records, &root, [outside.as_path()]).is_empty());
}
