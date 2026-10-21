use crate::{
    models::{HistoryOperation, HistoryOperationKind, HistoryOperationStatus},
    test_support::history::{history_operation, running_history_run},
    undo_redo::recorded_roots::RecordedRoots,
};

fn roots() -> RecordedRoots {
    RecordedRoots::of(&crate::models::HistoryRun {
        destination_root_path: Some("C:/todo/Processed".to_string()),
        selected_folder: "C:/todo".to_string(),
        ..running_history_run("run-1", "run-1.jsonl")
    })
}

fn operation(kind: HistoryOperationKind, source: &str, destination: &str) -> HistoryOperation {
    HistoryOperation {
        source_path: source.to_string(),
        destination_path: destination.to_string(),
        ..history_operation("run-1", "op-1", "file-1", kind)
    }
}

#[test]
fn accepts_recorded_paths_inside_the_run_folders_in_either_direction() {
    let original = "C:/todo/photo.jpg";
    let processed = "C:/todo/Processed/2026/photo.jpg";

    assert!(roots()
        .check(&operation(
            HistoryOperationKind::Process,
            original,
            processed
        ))
        .is_ok());
    assert!(roots()
        .check(&operation(HistoryOperationKind::Undo, processed, original))
        .is_ok());
}

#[test]
fn blocks_recorded_paths_that_leave_the_run_folders() {
    let processed = "C:/todo/Processed/photo.jpg";
    let escapes = [
        operation(
            HistoryOperationKind::Process,
            "C:/other/photo.jpg",
            processed,
        ),
        operation(
            HistoryOperationKind::Process,
            "C:/todo/photo.jpg",
            "C:/elsewhere/photo.jpg",
        ),
        operation(
            HistoryOperationKind::Process,
            "C:/todo/../etc/photo.jpg",
            processed,
        ),
        operation(HistoryOperationKind::Redo, processed, processed),
    ];

    for escape in escapes {
        let blocked = roots().check(&escape).expect_err("must block");
        assert!(matches!(blocked.status, HistoryOperationStatus::Blocked));
    }
}
