use std::collections::{HashMap, HashSet};

use crate::models::{
    FileKind, FileRecord, MovePlan, MoveResult, MoveResultStatus, MoveStrategy, PlanStatus,
    ScanStatus,
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
    use crate::models::{ConflictStatus, MovePlan};

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
}
