use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::models::{
    ConflictStatus, DateKind, FileKind, FileRecord, MetadataResult, MoveConfirmationSummary,
    MovePlan, MoveResult, MoveResultStatus, MoveStrategy, PlanStatus, ScanStatus,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveOperation {
    pub file_id: String,
    pub source_path: String,
    pub destination_path: String,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedMoves {
    pub operations: Vec<MoveOperation>,
    pub skipped: Vec<MoveResult>,
}

#[derive(Debug, Clone, Default)]
pub struct RecheckedMoves {
    pub ready: Vec<MoveOperation>,
    pub skipped: Vec<MoveResult>,
}

pub fn prepare_move_operations(files: &[FileRecord], plans: &[MovePlan]) -> PreparedMoves {
    let files_by_id: HashMap<&str, &FileRecord> =
        files.iter().map(|file| (file.id.as_str(), file)).collect();
    let mut seen_file_ids: HashSet<&str> = HashSet::new();
    let mut operations = Vec::new();
    let mut skipped = Vec::new();

    for plan in plans {
        let file_id = plan.file_id.as_str();
        if !seen_file_ids.insert(file_id) {
            skipped.push(skipped_result(
                &plan.file_id,
                "",
                &plan.destination_path,
                "Duplicate move plan for the same file id.",
            ));
            continue;
        }

        let Some(file) = files_by_id.get(file_id) else {
            skipped.push(skipped_result(
                &plan.file_id,
                "",
                &plan.destination_path,
                "No source file record found for move plan.",
            ));
            continue;
        };

        if !matches!(plan.plan_status, PlanStatus::Ready) {
            skipped.push(skipped_result(
                &plan.file_id,
                &file.source_path,
                &plan.destination_path,
                "Plan is not ready for execution.",
            ));
            continue;
        }

        if !matches!(file.scan_status, ScanStatus::Discovered) {
            skipped.push(skipped_result(
                &plan.file_id,
                &file.source_path,
                &plan.destination_path,
                "File was not discovered as a movable item.",
            ));
            continue;
        }

        if !matches!(file.kind, FileKind::Photo | FileKind::Video) {
            skipped.push(skipped_result(
                &plan.file_id,
                &file.source_path,
                &plan.destination_path,
                "Only photo/video files can be moved.",
            ));
            continue;
        }

        if plan.destination_path.is_empty() {
            skipped.push(skipped_result(
                &plan.file_id,
                &file.source_path,
                &plan.destination_path,
                "Plan is missing destination path.",
            ));
            continue;
        }

        operations.push(MoveOperation {
            file_id: plan.file_id.clone(),
            source_path: file.source_path.clone(),
            destination_path: plan.destination_path.clone(),
        });
    }

    PreparedMoves {
        operations,
        skipped,
    }
}

pub fn build_move_confirmation_summary(
    files: &[FileRecord],
    metadata_results: &[MetadataResult],
    plans: &[MovePlan],
) -> MoveConfirmationSummary {
    let prepared = prepare_move_operations(files, plans);
    let file_by_id: HashMap<&str, &FileRecord> =
        files.iter().map(|file| (file.id.as_str(), file)).collect();
    let metadata_by_id: HashMap<&str, &MetadataResult> = metadata_results
        .iter()
        .map(|metadata| (metadata.file_id.as_str(), metadata))
        .collect();

    let mut blocked_no_date = 0u64;
    let mut blocked_conflict = 0u64;
    let mut plan_errors = 0u64;
    let mut suffix_applied = 0u64;
    let mut fallback_warnings = 0u64;
    let mut photos_ready = 0u64;
    let mut videos_ready = 0u64;

    for plan in plans {
        match plan.plan_status {
            PlanStatus::BlockedNoDate => blocked_no_date += 1,
            PlanStatus::BlockedConflict => blocked_conflict += 1,
            PlanStatus::Error => plan_errors += 1,
            PlanStatus::Ready => {}
        }

        if matches!(plan.conflict_status, ConflictStatus::SuffixApplied) {
            suffix_applied += 1;
        }
    }

    for operation in &prepared.operations {
        if let Some(file) = file_by_id.get(operation.file_id.as_str()) {
            match file.kind {
                FileKind::Photo => photos_ready += 1,
                FileKind::Video => videos_ready += 1,
                _ => {}
            }
        }
        if let Some(metadata) = metadata_by_id.get(operation.file_id.as_str()) {
            if matches!(metadata.date_kind, Some(DateKind::Fallback)) {
                fallback_warnings += 1;
            }
        }
    }

    MoveConfirmationSummary {
        total_plans: plans.len() as u64,
        ready_operations: prepared.operations.len() as u64,
        skipped_operations: prepared.skipped.len() as u64,
        blocked_no_date,
        blocked_conflict,
        plan_errors,
        suffix_applied,
        fallback_warnings,
        photos_ready,
        videos_ready,
    }
}

pub fn recheck_destination_conflicts(operations: Vec<MoveOperation>) -> RecheckedMoves {
    let mut ready = Vec::new();
    let mut skipped = Vec::new();

    for operation in operations {
        if Path::new(&operation.destination_path).exists() {
            skipped.push(skipped_result(
                &operation.file_id,
                &operation.source_path,
                &operation.destination_path,
                "Destination file already exists at execution time.",
            ));
            continue;
        }

        ready.push(operation);
    }

    RecheckedMoves { ready, skipped }
}

fn skipped_result(
    file_id: &str,
    source_path: &str,
    destination_path: &str,
    reason: &str,
) -> MoveResult {
    MoveResult {
        file_id: file_id.to_string(),
        attempted_source_path: source_path.to_string(),
        attempted_destination_path: destination_path.to_string(),
        move_strategy: MoveStrategy::NotAttempted,
        result_status: MoveResultStatus::Skipped,
        error: Some(reason.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::models::{ConflictStatus, DateKind, MetadataResult, MetadataStatus, MovePlan};
    use tempfile::tempdir;

    use super::*;

    fn file_record(id: &str, kind: FileKind) -> FileRecord {
        FileRecord {
            id: id.to_string(),
            source_path: format!(r"C:\todo\{id}.jpg"),
            relative_path: format!(r"{id}.jpg"),
            extension: "jpg".to_string(),
            original_name: id.to_string(),
            kind,
            scan_status: ScanStatus::Discovered,
        }
    }

    fn ready_plan(id: &str, destination_path: &str) -> MovePlan {
        MovePlan {
            file_id: id.to_string(),
            original_name: id.to_string(),
            destination_dir: r"C:\todo\Processed\2026\05\23".to_string(),
            destination_filename: "2026-05-23 10-00-00 (IMG).jpg".to_string(),
            destination_path: destination_path.to_string(),
            conflict_status: ConflictStatus::None,
            plan_status: PlanStatus::Ready,
        }
    }

    #[test]
    fn creates_operation_for_ready_media_plan() {
        let files = vec![file_record("IMG_1001", FileKind::Photo)];
        let plans = vec![ready_plan(
            "IMG_1001",
            r"C:\todo\Processed\2026\05\23\2026-05-23 10-00-00 (IMG_1001).jpg",
        )];

        let prepared = prepare_move_operations(&files, &plans);

        assert_eq!(prepared.operations.len(), 1);
        assert_eq!(prepared.skipped.len(), 0);
        assert_eq!(prepared.operations[0].file_id, "IMG_1001");
    }

    #[test]
    fn skips_when_plan_not_ready() {
        let files = vec![file_record("IMG_1001", FileKind::Photo)];
        let mut plan = ready_plan("IMG_1001", r"C:\todo\Processed\2026\05\23\output.jpg");
        plan.plan_status = PlanStatus::BlockedNoDate;

        let prepared = prepare_move_operations(&files, &[plan]);

        assert_eq!(prepared.operations.len(), 0);
        assert_eq!(prepared.skipped.len(), 1);
        assert!(matches!(
            prepared.skipped[0].result_status,
            MoveResultStatus::Skipped
        ));
        assert!(matches!(
            prepared.skipped[0].move_strategy,
            MoveStrategy::NotAttempted
        ));
    }

    #[test]
    fn skips_unsupported_file_kind() {
        let files = vec![file_record("note", FileKind::Unsupported)];
        let plans = vec![ready_plan("note", r"C:\todo\Processed\2026\05\23\note.jpg")];

        let prepared = prepare_move_operations(&files, &plans);

        assert_eq!(prepared.operations.len(), 0);
        assert_eq!(prepared.skipped.len(), 1);
        assert_eq!(prepared.skipped[0].file_id, "note");
    }

    fn ready_metadata(file_id: &str, date_kind: DateKind) -> MetadataResult {
        MetadataResult {
            file_id: file_id.to_string(),
            chosen_date: Some("2026-05-23T10:00:00+10:00".to_string()),
            chosen_date_source: Some("DateTimeOriginal".to_string()),
            date_kind: Some(date_kind),
            raw_metadata_date: Some("2026:05:23 10:00:00".to_string()),
            metadata_status: MetadataStatus::Ready,
            error: None,
        }
    }

    #[test]
    fn builds_confirmation_summary_for_ready_and_blocked_items() {
        let files = vec![
            file_record("photo_1", FileKind::Photo),
            file_record("video_1", FileKind::Video),
        ];
        let mut blocked = ready_plan("video_1", r"C:\todo\Processed\2026\05\23\video_1.jpg");
        blocked.plan_status = PlanStatus::BlockedNoDate;
        let plans = vec![
            ready_plan("photo_1", r"C:\todo\Processed\2026\05\23\photo_1.jpg"),
            blocked,
        ];
        let metadata = vec![
            ready_metadata("photo_1", DateKind::Fallback),
            ready_metadata("video_1", DateKind::TimezoneAware),
        ];

        let summary = build_move_confirmation_summary(&files, &metadata, &plans);

        assert_eq!(summary.total_plans, 2);
        assert_eq!(summary.ready_operations, 1);
        assert_eq!(summary.skipped_operations, 1);
        assert_eq!(summary.blocked_no_date, 1);
        assert_eq!(summary.photos_ready, 1);
        assert_eq!(summary.videos_ready, 0);
        assert_eq!(summary.fallback_warnings, 1);
    }

    #[test]
    fn counts_suffix_applied_conflicts() {
        let files = vec![file_record("photo_1", FileKind::Photo)];
        let mut plan = ready_plan("photo_1", r"C:\todo\Processed\2026\05\23\photo_1 - 2.jpg");
        plan.conflict_status = ConflictStatus::SuffixApplied;
        let metadata = vec![ready_metadata("photo_1", DateKind::TimezoneAware)];

        let summary = build_move_confirmation_summary(&files, &metadata, &[plan]);

        assert_eq!(summary.suffix_applied, 1);
        assert_eq!(summary.ready_operations, 1);
        assert_eq!(summary.skipped_operations, 0);
    }

    #[test]
    fn recheck_skips_operation_when_destination_exists() {
        let temp = tempdir().expect("temp dir");
        let source = temp.path().join("source.jpg");
        let destination = temp.path().join("destination.jpg");
        fs::write(&source, b"source").expect("write source");
        fs::write(&destination, b"existing").expect("write destination");

        let operations = vec![MoveOperation {
            file_id: "photo_1".to_string(),
            source_path: source.to_string_lossy().to_string(),
            destination_path: destination.to_string_lossy().to_string(),
        }];

        let rechecked = recheck_destination_conflicts(operations);

        assert_eq!(rechecked.ready.len(), 0);
        assert_eq!(rechecked.skipped.len(), 1);
        assert_eq!(rechecked.skipped[0].file_id, "photo_1");
        assert!(matches!(
            rechecked.skipped[0].result_status,
            MoveResultStatus::Skipped
        ));
    }

    #[test]
    fn recheck_keeps_operation_when_destination_is_available() {
        let temp = tempdir().expect("temp dir");
        let source = temp.path().join("source.jpg");
        let destination = temp.path().join("destination.jpg");
        fs::write(&source, b"source").expect("write source");

        let operations = vec![MoveOperation {
            file_id: "photo_1".to_string(),
            source_path: source.to_string_lossy().to_string(),
            destination_path: destination.to_string_lossy().to_string(),
        }];

        let rechecked = recheck_destination_conflicts(operations);

        assert_eq!(rechecked.ready.len(), 1);
        assert_eq!(rechecked.skipped.len(), 0);
        assert_eq!(rechecked.ready[0].file_id, "photo_1");
    }
}
