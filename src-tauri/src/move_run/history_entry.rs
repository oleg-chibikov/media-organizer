use std::{collections::HashMap, fs};

use chrono::Utc;

use super::{request::RunLookups, status_mapping::history_operation_status_for_move_result};
use crate::{
    history,
    models::{FileRecord, HistoryOperation, HistoryOperationKind, MoveResult, MoveResultStatus},
};

/// Captures the source size once per file, before any move can change it.
pub(super) fn ensure_expected_source_size<'a>(
    result: &MoveResult,
    files_by_id: &HashMap<&'a str, &'a FileRecord>,
    expected_source_sizes: &mut HashMap<&'a str, Option<u64>>,
) {
    let Some(file) = files_by_id.get(result.file_id.as_str()) else {
        return;
    };

    expected_source_sizes
        .entry(file.id.as_str())
        .or_insert_with(|| fs::metadata(&file.source_path).ok().map(|meta| meta.len()));
}

pub(super) fn build_history_operation_for_result(
    run_id: &str,
    result: &MoveResult,
    lookups: &RunLookups,
    expected_source_sizes: &HashMap<&str, Option<u64>>,
) -> HistoryOperation {
    let file_id = result.file_id.as_str();
    let metadata_date = lookups
        .metadata_by_id
        .get(file_id)
        .and_then(|metadata| metadata.chosen_date.clone());
    let original_name = lookups
        .plans_by_id
        .get(file_id)
        .map(|plan| plan.original_name.clone())
        .or_else(|| {
            lookups
                .files_by_id
                .get(file_id)
                .map(|file| file.original_name.clone())
        })
        .unwrap_or_else(|| result.file_id.clone());
    let expected_source_size = expected_source_sizes.get(file_id).copied().flatten();

    HistoryOperation {
        operation_id: history::next_operation_id(run_id),
        run_id: run_id.to_string(),
        file_id: result.file_id.clone(),
        operation_kind: HistoryOperationKind::Process,
        source_path: result.attempted_source_path.clone(),
        destination_path: result.attempted_destination_path.clone(),
        expected_source_size,
        expected_destination_size: expected_destination_size_from_result(result),
        metadata_date,
        original_name,
        move_strategy: result.move_strategy.clone(),
        status: history_operation_status_for_move_result(&result.result_status),
        error: result.error.clone(),
        timestamp: Utc::now().to_rfc3339(),
    }
}

fn expected_destination_size_from_result(result: &MoveResult) -> Option<u64> {
    match result.result_status {
        MoveResultStatus::Moved | MoveResultStatus::Copied => {
            fs::metadata(&result.attempted_destination_path)
                .ok()
                .map(|meta| meta.len())
        }
        MoveResultStatus::Skipped | MoveResultStatus::Failed => None,
    }
}
