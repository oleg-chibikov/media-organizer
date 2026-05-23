use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;

use crate::events;
use crate::filename;
use crate::models::{
    ConflictStatus, FileKind, FileRecord, MetadataResult, MetadataStatus, MovePlan, PlanErrorEvent,
    PlanFileEvent, PlanFinishedEvent, PlanStatus,
};

pub fn stream_move_plan(
    app: &tauri::AppHandle,
    selected_folder: PathBuf,
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
) -> Result<(), String> {
    let metadata_by_id: HashMap<&str, &MetadataResult> = metadata_results
        .iter()
        .map(|result| (result.file_id.as_str(), result))
        .collect();
    let mut reserved_paths: HashSet<String> = HashSet::new();
    let mut planned = 0_u64;
    let mut errors = 0_u64;

    for file in files {
        if !matches!(file.kind, FileKind::Photo | FileKind::Video) {
            continue;
        }

        let metadata = metadata_by_id.get(file.id.as_str()).copied();
        match create_plan_for_file(&selected_folder, &file, metadata, &mut reserved_paths) {
            Ok(plan) => {
                planned += 1;
                events::emit_plan_file(app, PlanFileEvent { plan })?;
            }
            Err(error) => {
                errors += 1;
                events::emit_plan_error(
                    app,
                    PlanErrorEvent {
                        file_id: file.id.clone(),
                        error,
                    },
                )?;
            }
        }
    }

    events::emit_plan_finished(app, PlanFinishedEvent { planned, errors })
}

fn create_plan_for_file(
    selected_folder: &Path,
    file: &FileRecord,
    metadata: Option<&MetadataResult>,
    reserved_paths: &mut HashSet<String>,
) -> Result<MovePlan, String> {
    if !matches!(file.scan_status, crate::models::ScanStatus::Discovered) {
        return Ok(empty_plan(file, PlanStatus::BlockedNoDate));
    }

    let Some(metadata) = metadata else {
        return Ok(empty_plan(file, PlanStatus::BlockedNoDate));
    };

    if !matches!(metadata.metadata_status, MetadataStatus::Ready) {
        return Ok(empty_plan(file, PlanStatus::BlockedNoDate));
    }

    let Some(chosen_date) = metadata.chosen_date.as_deref() else {
        return Ok(empty_plan(file, PlanStatus::BlockedNoDate));
    };

    let parsed_date = NaiveDateTime::parse_from_str(chosen_date, "%Y-%m-%d %H:%M:%S")
        .map_err(|error| format!("Invalid chosen date for {}: {error}", file.source_path))?;

    let destination_dir = selected_folder
        .join("Processed")
        .join(parsed_date.format("%Y").to_string())
        .join(parsed_date.format("%m").to_string())
        .join(parsed_date.format("%d").to_string());

    let original_name = filename::sanitize_for_windows_filename(&file.original_name);
    let stem_base = format!("{} ({original_name})", parsed_date.format("%Y-%m-%d %H-%M-%S"));

    let extension = file.extension.clone();
    let (destination_path, destination_filename, conflict_status) =
        allocate_destination_path(&destination_dir, &stem_base, &extension, reserved_paths);

    Ok(MovePlan {
        file_id: file.id.clone(),
        original_name,
        destination_dir: destination_dir.to_string_lossy().to_string(),
        destination_filename,
        destination_path: destination_path.to_string_lossy().to_string(),
        conflict_status,
        plan_status: PlanStatus::Ready,
    })
}

fn allocate_destination_path(
    destination_dir: &Path,
    stem_base: &str,
    extension: &str,
    reserved_paths: &mut HashSet<String>,
) -> (PathBuf, String, ConflictStatus) {
    let mut suffix = 1_u32;
    loop {
        let stem = if suffix == 1 {
            stem_base.to_string()
        } else {
            format!("{stem_base} - {suffix}")
        };

        let filename = if extension.is_empty() {
            stem.clone()
        } else {
            format!("{stem}.{extension}")
        };
        let destination_path = destination_dir.join(&filename);
        let path_key = normalize_path_key(&destination_path);

        let exists_on_disk = destination_path.exists();
        let exists_in_run = reserved_paths.contains(&path_key);

        if !exists_on_disk && !exists_in_run {
            reserved_paths.insert(path_key);
            let conflict_status = if suffix == 1 {
                ConflictStatus::None
            } else {
                ConflictStatus::SuffixApplied
            };
            return (destination_path, filename, conflict_status);
        }

        suffix += 1;
    }
}

fn normalize_path_key(path: &Path) -> String {
    path.to_string_lossy().to_ascii_lowercase()
}

fn empty_plan(file: &FileRecord, plan_status: PlanStatus) -> MovePlan {
    MovePlan {
        file_id: file.id.clone(),
        original_name: filename::sanitize_for_windows_filename(&file.original_name),
        destination_dir: String::new(),
        destination_filename: String::new(),
        destination_path: String::new(),
        conflict_status: ConflictStatus::Blocked,
        plan_status,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::Path;

    use super::{allocate_destination_path, create_plan_for_file};
    use crate::models::{
        FileKind, FileRecord, MetadataResult, MetadataStatus, PlanStatus, ScanStatus,
    };

    fn file_record(id: &str, extension: &str) -> FileRecord {
        FileRecord {
            id: id.to_string(),
            source_path: format!(r"C:\todo\{id}.{extension}"),
            relative_path: format!(r"{id}.{extension}"),
            extension: extension.to_string(),
            original_name: id.to_string(),
            kind: FileKind::Photo,
            scan_status: ScanStatus::Discovered,
        }
    }

    fn metadata_result(file_id: &str, chosen_date: &str) -> MetadataResult {
        MetadataResult {
            file_id: file_id.to_string(),
            chosen_date: Some(chosen_date.to_string()),
            chosen_date_source: Some("DateTimeOriginal".to_string()),
            date_kind: None,
            raw_metadata_date: Some(chosen_date.to_string()),
            metadata_status: MetadataStatus::Ready,
            error: None,
        }
    }

    #[test]
    fn plans_processed_destination_and_filename() {
        let mut reserved = HashSet::new();
        let file = file_record("IMG_1001", "mp4");
        let metadata = metadata_result("IMG_1001", "2026-03-05 07:20:17");

        let plan = create_plan_for_file(Path::new(r"C:\todo"), &file, Some(&metadata), &mut reserved)
            .expect("plan should be created");

        assert!(plan.destination_dir.ends_with(r"Processed\2026\03\05"));
        assert_eq!(
            plan.destination_filename,
            "2026-03-05 07-20-17 (IMG_1001).mp4"
        );
    }

    #[test]
    fn applies_suffix_for_existing_destination_file() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let destination_dir = temp_dir.path().join("Processed").join("2026").join("03").join("05");
        std::fs::create_dir_all(&destination_dir).expect("destination directory should be created");
        std::fs::write(
            destination_dir.join("2026-03-05 07-20-17 (IMG_1001).mp4"),
            b"existing",
        )
        .expect("existing file should be created");

        let mut reserved = HashSet::new();
        let file = file_record("IMG_1001", "mp4");
        let metadata = metadata_result("IMG_1001", "2026-03-05 07:20:17");

        let plan = create_plan_for_file(temp_dir.path(), &file, Some(&metadata), &mut reserved)
            .expect("plan should be created");

        assert_eq!(
            plan.destination_filename,
            "2026-03-05 07-20-17 (IMG_1001) - 2.mp4"
        );
    }

    #[test]
    fn applies_suffix_for_duplicate_in_same_run() {
        let destination_dir = Path::new(r"C:\todo\Processed\2026\03\05");
        let stem_base = "2026-03-05 07-20-17 (IMG_1001)";
        let mut reserved = HashSet::new();

        let (_, filename_one, _) =
            allocate_destination_path(destination_dir, stem_base, "mp4", &mut reserved);
        let (_, filename_two, _) =
            allocate_destination_path(destination_dir, stem_base, "mp4", &mut reserved);

        assert_eq!(filename_one, "2026-03-05 07-20-17 (IMG_1001).mp4");
        assert_eq!(filename_two, "2026-03-05 07-20-17 (IMG_1001) - 2.mp4");
    }

    #[test]
    fn blocks_plan_when_metadata_not_ready() {
        let mut reserved = HashSet::new();
        let file = file_record("IMG_1001", "jpg");

        let plan =
            create_plan_for_file(Path::new(r"C:\todo"), &file, None, &mut reserved).unwrap();

        assert!(matches!(plan.plan_status, PlanStatus::BlockedNoDate));
        assert!(plan.destination_path.is_empty());
    }

    #[test]
    fn dry_run_planning_does_not_create_destination_directories() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let mut reserved = HashSet::new();
        let file = file_record("IMG_1001", "jpg");
        let metadata = metadata_result("IMG_1001", "2026-03-05 07:20:17");

        let plan = create_plan_for_file(temp_dir.path(), &file, Some(&metadata), &mut reserved)
            .expect("plan should be created");

        assert!(matches!(plan.plan_status, PlanStatus::Ready));
        let destination_dir = temp_dir.path().join("Processed").join("2026").join("03").join("05");
        assert!(
            !destination_dir.exists(),
            "planner dry-run must not create filesystem directories"
        );
    }
}
