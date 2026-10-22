use std::{fs, path::Path};

use tempfile::tempdir;
use tokio_util::sync::CancellationToken;

use crate::{
    events,
    history::HistoryStore,
    models::{DateKind, DestinationRootMode, EffectiveDestinationRoot, FileKind, FileRecord},
    move_run::{
        execution::run_move_plan,
        request::{MoveBatch, MoveRunRequest},
    },
    placements::PlacementStore,
    test_support::{
        events::RecordingSink,
        media::{file_record, ready_metadata, ready_plan},
    },
};

fn photo(id: &str, source: &Path) -> FileRecord {
    FileRecord {
        source_path: source.to_string_lossy().to_string(),
        ..file_record(id, "jpg", FileKind::Photo)
    }
}

#[test]
fn stops_moving_files_once_the_history_log_fails() {
    let temp = tempdir().expect("temp dir");
    let selected = temp.path().join("todo");
    let root = selected.join("Processed");
    let (first, second) = (selected.join("a.jpg"), selected.join("b.jpg"));
    let (first_target, second_target) = (root.join("a.jpg"), root.join("b.jpg"));
    fs::create_dir_all(&selected).expect("create selected folder");
    fs::write(&first, b"first").expect("write first");
    fs::write(&second, b"second").expect("write second");
    let effective_root = EffectiveDestinationRoot {
        mode: DestinationRootMode::ProcessedFolder,
        root_path: root.to_string_lossy().to_string(),
        custom_root_configured: false,
        force_processed_folder: false,
        warning: None,
    };
    let history = HistoryStore::in_app_data(&temp.path().join("app-data"));
    let mut history_run = history
        .start_run(&selected, &effective_root, &root)
        .expect("start history run");
    history_run.break_log_for_tests();
    let request = MoveRunRequest {
        selected_folder: selected.clone(),
        destination_root: root.clone(),
        place_root: root.clone(),
        batch: MoveBatch {
            files: vec![photo("a", &first), photo("b", &second)],
            metadata_results: vec![
                ready_metadata("a", DateKind::TimezoneAware),
                ready_metadata("b", DateKind::TimezoneAware),
            ],
            plans: vec![
                ready_plan("a", &first_target.to_string_lossy()),
                ready_plan("b", &second_target.to_string_lossy()),
            ],
        },
        placements: PlacementStore::in_app_data(&temp.path().join("app-data")),
    };
    let sink = RecordingSink::default();

    run_move_plan(&sink, request, history_run, &CancellationToken::new());

    assert!(first_target.exists() && !first.exists());
    assert!(second.exists() && !second_target.exists());
    let finished = &sink.payloads(events::MOVE_FINISHED)[0];
    assert_eq!(finished["moved"], 1);
    assert_eq!(finished["skipped"], 1);
}
