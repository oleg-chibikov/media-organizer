use crate::models::{
    HistoryIndexEntry, HistoryOperation, HistoryOperationKind, HistoryOperationStatus, HistoryRun,
    HistoryRunStatus, MoveStrategy,
};

/// Succeeded rename operation; tests override paths, sizes and status as needed.
pub(crate) fn history_operation(
    run_id: &str,
    operation_id: &str,
    file_id: &str,
    operation_kind: HistoryOperationKind,
) -> HistoryOperation {
    HistoryOperation {
        operation_id: operation_id.to_string(),
        run_id: run_id.to_string(),
        file_id: file_id.to_string(),
        operation_kind,
        source_path: r"C:\todo\photo.jpg".to_string(),
        destination_path: r"C:\todo\Processed\photo.jpg".to_string(),
        expected_source_size: Some(10),
        expected_destination_size: Some(10),
        metadata_date: None,
        original_name: "photo".to_string(),
        move_strategy: MoveStrategy::Rename,
        status: HistoryOperationStatus::Succeeded,
        error: None,
        timestamp: "2026-05-24T01:03:03Z".to_string(),
    }
}

pub(crate) fn running_history_run(run_id: &str, history_file_path: &str) -> HistoryRun {
    HistoryRun {
        run_id: run_id.to_string(),
        history_version: crate::history::HISTORY_VERSION,
        app_version: "0.1.0".to_string(),
        selected_folder: r"C:\todo".to_string(),
        folder_label: "todo".to_string(),
        destination_root_mode: None,
        destination_root_path: None,
        place_root_path: None,
        started_at: "2026-05-24T01:02:03Z".to_string(),
        completed_at: None,
        run_status: HistoryRunStatus::Running,
        history_file_path: history_file_path.to_string(),
    }
}

pub(crate) fn running_index_entry(
    run_id: &str,
    started_at: &str,
    history_file_path: &str,
) -> HistoryIndexEntry {
    HistoryIndexEntry {
        run_id: run_id.to_string(),
        folder_label: "todo".to_string(),
        selected_folder: r"C:\todo".to_string(),
        destination_root_path: None,
        started_at: started_at.to_string(),
        completed_at: None,
        run_status: HistoryRunStatus::Running,
        files_migrated: 0,
        undo_available: false,
        redo_available: false,
        history_file_path: history_file_path.to_string(),
    }
}
