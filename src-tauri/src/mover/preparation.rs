use std::collections::{HashMap, HashSet};

use super::{outcome::skipped_result, MoveOperation, PreparedMoves};
use crate::models::{FileKind, FileRecord, MovePlan, MoveResult, PlanStatus, ScanStatus};

/// Turns ready plans for discovered photo/video files into move operations.
pub(crate) fn prepare_move_operations(files: &[FileRecord], plans: &[MovePlan]) -> PreparedMoves {
    let files_by_id: HashMap<&str, &FileRecord> =
        files.iter().map(|file| (file.id.as_str(), file)).collect();
    let mut seen_file_ids: HashSet<&str> = HashSet::new();
    let mut prepared = PreparedMoves::default();

    for plan in plans {
        match operation_for_plan(plan, &files_by_id, &mut seen_file_ids) {
            Ok(operation) => prepared.operations.push(operation),
            Err(skipped) => prepared.skipped.push(skipped),
        }
    }

    prepared
}

fn operation_for_plan<'a>(
    plan: &'a MovePlan,
    files_by_id: &HashMap<&str, &FileRecord>,
    seen_file_ids: &mut HashSet<&'a str>,
) -> Result<MoveOperation, MoveResult> {
    let skip = |source_path: &str, reason: &str| {
        skipped_result(&plan.file_id, source_path, &plan.destination_path, reason)
    };

    if !seen_file_ids.insert(plan.file_id.as_str()) {
        return Err(skip("", "Duplicate move plan for the same file id."));
    }
    let Some(file) = files_by_id.get(plan.file_id.as_str()) else {
        return Err(skip("", "No source file record found for move plan."));
    };
    if !matches!(plan.plan_status, PlanStatus::Ready) {
        return Err(skip(&file.source_path, "Plan is not ready for execution."));
    }
    if !matches!(file.scan_status, ScanStatus::Discovered) {
        return Err(skip(
            &file.source_path,
            "File was not discovered as a movable item.",
        ));
    }
    if !matches!(file.kind, FileKind::Photo | FileKind::Video) {
        return Err(skip(
            &file.source_path,
            "Only photo/video files can be moved.",
        ));
    }
    if plan.destination_path.is_empty() {
        return Err(skip(&file.source_path, "Plan is missing destination path."));
    }

    Ok(MoveOperation {
        file_id: plan.file_id.clone(),
        source_path: file.source_path.clone(),
        destination_path: plan.destination_path.clone(),
    })
}
