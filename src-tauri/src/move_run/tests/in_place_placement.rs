use std::{fs, path::Path};

use tempfile::tempdir;
use tokio_util::sync::CancellationToken;

use crate::{
    history::HistoryStore,
    models::{FileKind, FilePlace, FileRecord, HomeMatch, MovePlan},
    move_run::{start_move_run, MoveBatch},
    settings::{set_rename_in_place, set_root_media_folder, SettingsStore},
    test_support::{
        events::RecordingSink,
        media::{dated_metadata, file_record, ready_plan},
    },
    undo_redo::{prepare_replay, ReplayDirection},
};

fn photo(id: &str, source: &Path) -> FileRecord {
    FileRecord {
        source_path: source.to_string_lossy().to_string(),
        ..file_record(id, "jpg", FileKind::Photo)
    }
}

fn plan(id: &str, destination: &Path, placed: bool) -> MovePlan {
    MovePlan {
        place: placed.then(|| FilePlace {
            place: "Zoo".to_owned(),
            country: "Australia".to_owned(),
            home: HomeMatch::Country,
        }),
        ..ready_plan(id, &destination.to_string_lossy())
    }
}

#[test]
fn renaming_in_place_moves_placed_files_to_the_root_and_undoes_them() {
    let temp = tempdir().expect("temp dir");
    let app_data = temp.path().join("app-data");
    let (selected, root) = (temp.path().join("2025-03"), temp.path().join("Media"));
    let (placed, renamed) = (selected.join("a.jpg"), selected.join("b.jpg"));
    let zoo = root.join("2025").join("Zoo").join("a.jpg");
    let renamed_to = selected.join("2025-03-01 b.jpg");
    fs::create_dir_all(&selected).expect("selected folder");
    fs::create_dir_all(&root).expect("root folder");
    fs::write(&placed, b"a").expect("write placed");
    fs::write(&renamed, b"b").expect("write renamed");
    let settings = SettingsStore::in_app_data(&app_data);
    let root_path = root.to_string_lossy().to_string();
    settings
        .update(|current| {
            set_rename_in_place(&set_root_media_folder(current, Some(root_path)), true)
        })
        .expect("settings");
    let batch = MoveBatch {
        files: vec![photo("a", &placed), photo("b", &renamed)],
        metadata_results: vec![
            dated_metadata("a", "2025-03-01 10:00:00"),
            dated_metadata("b", "2025-03-01 11:00:00"),
        ],
        plans: vec![plan("a", &zoo, true), plan("b", &renamed_to, false)],
    };
    let sink = RecordingSink::default();
    let history = HistoryStore::in_app_data(&app_data);

    start_move_run(&sink, &settings, &history, &selected, batch)
        .expect("run starts")
        .execute(&sink, &CancellationToken::new());

    assert!(zoo.exists() && renamed_to.exists());

    let run_id = history.list_runs().expect("runs")[0].run_id.clone();
    prepare_replay(&history, &run_id, ReplayDirection::Undo)
        .expect("undoable")
        .run(&sink);

    assert!(placed.exists() && renamed.exists());
    assert!(!zoo.exists() && !renamed_to.exists());
}
