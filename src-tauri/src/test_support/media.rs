use std::path::Path;

use crate::{
    models::{
        ConflictStatus, DateKind, FileKind, FileRecord, MetadataResult, MetadataStatus, MovePlan,
        PlanStatus, ScanStatus,
    },
    mover::MoveOperation,
};

pub(crate) fn file_record(id: &str, extension: &str, kind: FileKind) -> FileRecord {
    FileRecord {
        id: id.to_string(),
        source_path: format!(r"C:\todo\{id}.{extension}"),
        relative_path: format!(r"{id}.{extension}"),
        extension: extension.to_string(),
        original_name: id.to_string(),
        kind,
        scan_status: ScanStatus::Discovered,
    }
}

pub(crate) fn ready_plan(id: &str, destination_path: &str) -> MovePlan {
    MovePlan {
        file_id: id.to_string(),
        original_name: id.to_string(),
        destination_dir: r"C:\todo\Processed\2026\05\23".to_string(),
        destination_filename: "2026-05-23 10-00-00 (IMG).jpg".to_string(),
        destination_path: destination_path.to_string(),
        conflict_status: ConflictStatus::None,
        plan_status: PlanStatus::Ready,
        placement: None,
    }
}

pub(crate) fn ready_metadata(file_id: &str, date_kind: DateKind) -> MetadataResult {
    MetadataResult {
        file_id: file_id.to_string(),
        chosen_date: Some("2026-05-23T10:00:00+10:00".to_string()),
        chosen_date_source: Some("DateTimeOriginal".to_string()),
        date_kind: Some(date_kind),
        raw_metadata_date: Some("2026:05:23 10:00:00".to_string()),
        device_name: None,
        gps: None,
        metadata_status: MetadataStatus::Ready,
        error: None,
    }
}

pub(crate) fn photo_move_operation(source: &Path, destination: &Path) -> MoveOperation {
    MoveOperation {
        file_id: "photo_1".to_string(),
        source_path: source.to_string_lossy().to_string(),
        destination_path: destination.to_string_lossy().to_string(),
    }
}

/// Ready metadata for a planner-format date captured on an iPhone 15 Pro.
pub(crate) fn dated_metadata(file_id: &str, chosen_date: &str) -> MetadataResult {
    MetadataResult {
        file_id: file_id.to_string(),
        chosen_date: Some(chosen_date.to_string()),
        chosen_date_source: Some("DateTimeOriginal".to_string()),
        date_kind: None,
        raw_metadata_date: Some(chosen_date.to_string()),
        device_name: Some("iPhone 15 Pro".to_string()),
        gps: None,
        metadata_status: MetadataStatus::Ready,
        error: None,
    }
}
