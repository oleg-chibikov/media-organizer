use std::collections::HashMap;

use super::prepare_move_operations;
use crate::models::{
    ConflictStatus, DateKind, FileKind, FileRecord, MetadataResult, MoveConfirmationSummary,
    MovePlan, PlanStatus,
};

/// Counts what a move would do. Files that already have their planned name are left out.
pub(crate) fn build_move_confirmation_summary(
    files: &[FileRecord],
    metadata_results: &[MetadataResult],
    plans: &[MovePlan],
) -> MoveConfirmationSummary {
    let plans: Vec<MovePlan> = plans
        .iter()
        .filter(|plan| !matches!(plan.plan_status, PlanStatus::AlreadyNamed))
        .cloned()
        .collect();
    let prepared = prepare_move_operations(files, &plans);
    let file_by_id: HashMap<&str, &FileRecord> =
        files.iter().map(|file| (file.id.as_str(), file)).collect();
    let metadata_by_id: HashMap<&str, &MetadataResult> = metadata_results
        .iter()
        .map(|metadata| (metadata.file_id.as_str(), metadata))
        .collect();

    let mut summary = MoveConfirmationSummary {
        total_plans: plans.len() as u64,
        ready_operations: prepared.operations.len() as u64,
        skipped_operations: prepared.skipped.len() as u64,
        blocked_no_date: 0,
        blocked_conflict: 0,
        plan_errors: 0,
        suffix_applied: 0,
        fallback_warnings: 0,
        photos_ready: 0,
        videos_ready: 0,
    };

    for plan in &plans {
        match plan.plan_status {
            PlanStatus::BlockedNoDate => summary.blocked_no_date += 1,
            PlanStatus::BlockedConflict => summary.blocked_conflict += 1,
            PlanStatus::Error => summary.plan_errors += 1,
            PlanStatus::Ready | PlanStatus::AlreadyNamed => {}
        }
        if matches!(plan.conflict_status, ConflictStatus::SuffixApplied) {
            summary.suffix_applied += 1;
        }
    }

    for operation in &prepared.operations {
        if let Some(file) = file_by_id.get(operation.file_id.as_str()) {
            match file.kind {
                FileKind::Photo => summary.photos_ready += 1,
                FileKind::Video => summary.videos_ready += 1,
                _ => {}
            }
        }
        if let Some(metadata) = metadata_by_id.get(operation.file_id.as_str()) {
            if matches!(metadata.date_kind, Some(DateKind::Fallback)) {
                summary.fallback_warnings += 1;
            }
        }
    }

    summary
}
