use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::{Datelike, NaiveDateTime};

use crate::events;
use crate::filename;
use crate::models::{
    ConflictStatus, FileKind, FileRecord, MetadataResult, MetadataStatus, MovePlan, PlanErrorEvent,
    PlanFileEvent, PlanFinishedEvent, PlanStatus,
};

#[allow(dead_code)]
pub fn stream_move_plan(
    app: &tauri::AppHandle,
    selected_folder: PathBuf,
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
) -> Result<(), String> {
    let destination_root = selected_folder.join("Processed");
    stream_move_plan_with_destination_root(app, destination_root, files, metadata_results)
}

pub fn stream_move_plan_with_destination_root(
    app: &tauri::AppHandle,
    destination_root: PathBuf,
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
        match create_plan_for_file(&destination_root, &file, metadata, &mut reserved_paths) {
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
    destination_root: &Path,
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

    let destination_dir = destination_root
        .join(parsed_date.format("%Y").to_string())
        .join(parsed_date.month().to_string())
        .join(parsed_date.day().to_string());

    let original_name = filename::sanitize_for_windows_filename(&file.original_name);
    let mut stem_base = format!(
        "{} ({original_name})",
        parsed_date.format("%Y-%m-%d %H-%M-%S")
    );
    if let Some(device_name) = metadata.device_name.as_deref() {
        let sanitized_device_name = filename::sanitize_for_windows_filename(device_name);
        if !sanitized_device_name.is_empty() {
            stem_base.push_str(" - ");
            stem_base.push_str(&sanitized_device_name);
        }
    }

    let extension = file.extension.clone();
    let (destination_path, destination_filename, conflict_status, plan_status) =
        allocate_destination_path(
            Path::new(&file.source_path),
            &destination_dir,
            &stem_base,
            &extension,
            reserved_paths,
        )?;

    Ok(MovePlan {
        file_id: file.id.clone(),
        original_name,
        destination_dir: destination_dir.to_string_lossy().to_string(),
        destination_filename,
        destination_path: destination_path.to_string_lossy().to_string(),
        conflict_status,
        plan_status,
    })
}

fn allocate_destination_path(
    source_path: &Path,
    destination_dir: &Path,
    stem_base: &str,
    extension: &str,
    reserved_paths: &mut HashSet<String>,
) -> Result<(PathBuf, String, ConflictStatus, PlanStatus), String> {
    let mut suffix = 0_u32;
    loop {
        let stem = if suffix == 0 {
            stem_base.to_string()
        } else {
            format!("{stem_base} ({suffix})")
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

        if exists_on_disk && suffix == 0 {
            if files_have_identical_contents(source_path, &destination_path)? {
                return Ok((
                    destination_path,
                    filename,
                    ConflictStatus::Blocked,
                    PlanStatus::BlockedConflict,
                ));
            }
        }

        if !exists_on_disk && !exists_in_run {
            reserved_paths.insert(path_key);
            let conflict_status = if suffix == 0 {
                ConflictStatus::None
            } else {
                ConflictStatus::SuffixApplied
            };
            return Ok((
                destination_path,
                filename,
                conflict_status,
                PlanStatus::Ready,
            ));
        }

        suffix += 1;
    }
}

fn files_have_identical_contents(
    source_path: &Path,
    destination_path: &Path,
) -> Result<bool, String> {
    let source_meta = source_path
        .metadata()
        .map_err(|error| format!("Failed to read source metadata for conflict check: {error}"))?;
    let destination_meta = destination_path.metadata().map_err(|error| {
        format!("Failed to read destination metadata for conflict check: {error}")
    })?;

    if source_meta.len() != destination_meta.len() {
        return Ok(false);
    }

    let mut source_file = File::open(source_path)
        .map_err(|error| format!("Failed to open source for conflict check: {error}"))?;
    let mut destination_file = File::open(destination_path)
        .map_err(|error| format!("Failed to open destination for conflict check: {error}"))?;
    let mut source_buffer = [0_u8; 64 * 1024];
    let mut destination_buffer = [0_u8; 64 * 1024];

    loop {
        let source_read = source_file
            .read(&mut source_buffer)
            .map_err(|error| format!("Failed reading source for conflict check: {error}"))?;
        let destination_read = destination_file
            .read(&mut destination_buffer)
            .map_err(|error| format!("Failed reading destination for conflict check: {error}"))?;

        if source_read != destination_read
            || source_buffer[..source_read] != destination_buffer[..destination_read]
        {
            return Ok(false);
        }

        if source_read == 0 {
            break;
        }
    }

    Ok(true)
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
    use std::path::{Path, PathBuf};

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
            device_name: Some("iPhone 15 Pro".to_string()),
            metadata_status: MetadataStatus::Ready,
            error: None,
        }
    }

    #[test]
    fn plans_forced_legacy_processed_root_destination_and_filename() {
        let mut reserved = HashSet::new();
        let file = file_record("IMG_1001", "mp4");
        let metadata = metadata_result("IMG_1001", "2026-03-05 07:20:17");

        let root = Path::new("todo").join("Processed");
        let plan = create_plan_for_file(&root, &file, Some(&metadata), &mut reserved)
            .expect("plan should be created");

        assert_eq!(
            PathBuf::from(&plan.destination_dir),
            root.join("2026").join("3").join("5")
        );
        assert_eq!(
            plan.destination_filename,
            "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro.mp4"
        );
    }

    #[test]
    fn plans_custom_root_destination_and_filename() {
        let mut reserved = HashSet::new();
        let file = file_record("IMG_1001", "mp4");
        let metadata = metadata_result("IMG_1001", "2026-03-05 07:20:17");

        let root = Path::new("MediaRoot");
        let plan = create_plan_for_file(root, &file, Some(&metadata), &mut reserved)
            .expect("plan should be created");

        assert_eq!(
            PathBuf::from(&plan.destination_dir),
            root.join("2026").join("3").join("5")
        );
        assert_eq!(
            plan.destination_filename,
            "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro.mp4"
        );
    }

    #[test]
    fn applies_suffix_for_existing_destination_file() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let source = temp_dir.path().join("IMG_1001.mp4");
        std::fs::write(&source, b"source-bytes").expect("source file should be created");
        let destination_dir = temp_dir.path().join("2026").join("3").join("5");
        std::fs::create_dir_all(&destination_dir).expect("destination directory should be created");
        std::fs::write(
            destination_dir.join("2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro.mp4"),
            b"existing",
        )
        .expect("existing file should be created");

        let mut reserved = HashSet::new();
        let mut file = file_record("IMG_1001", "mp4");
        file.source_path = source.to_string_lossy().to_string();
        let metadata = metadata_result("IMG_1001", "2026-03-05 07:20:17");

        let plan = create_plan_for_file(temp_dir.path(), &file, Some(&metadata), &mut reserved)
            .expect("plan should be created");

        assert_eq!(
            plan.destination_filename,
            "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro (1).mp4"
        );
    }

    #[test]
    fn blocks_conflict_when_existing_destination_has_identical_content() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let source = temp_dir.path().join("IMG_1001.mp4");
        std::fs::write(&source, b"same-bytes").expect("source file should be created");

        let destination_dir = temp_dir.path().join("2026").join("3").join("5");
        std::fs::create_dir_all(&destination_dir).expect("destination directory should be created");
        std::fs::write(
            destination_dir.join("2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro.mp4"),
            b"same-bytes",
        )
        .expect("existing file should be created");

        let mut reserved = HashSet::new();
        let mut file = file_record("IMG_1001", "mp4");
        file.source_path = source.to_string_lossy().to_string();
        let metadata = metadata_result("IMG_1001", "2026-03-05 07:20:17");

        let plan = create_plan_for_file(temp_dir.path(), &file, Some(&metadata), &mut reserved)
            .expect("plan should be created");

        assert!(matches!(plan.plan_status, PlanStatus::BlockedConflict));
        assert!(matches!(
            plan.conflict_status,
            crate::models::ConflictStatus::Blocked
        ));
        assert_eq!(
            plan.destination_filename,
            "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro.mp4"
        );
    }

    #[test]
    fn applies_suffix_for_duplicate_in_same_run() {
        let destination_dir = Path::new(r"C:\todo\Processed\2026\3\5");
        let stem_base = "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro";
        let mut reserved = HashSet::new();

        let source = Path::new(r"C:\todo\IMG_1001.mp4");
        let (_, filename_one, _, _) =
            allocate_destination_path(source, destination_dir, stem_base, "mp4", &mut reserved)
                .expect("first allocation should succeed");
        let (_, filename_two, _, _) =
            allocate_destination_path(source, destination_dir, stem_base, "mp4", &mut reserved)
                .expect("second allocation should succeed");

        assert_eq!(
            filename_one,
            "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro.mp4"
        );
        assert_eq!(
            filename_two,
            "2026-03-05 07-20-17 (IMG_1001) - iPhone 15 Pro (1).mp4"
        );
    }

    #[test]
    fn blocks_plan_when_metadata_not_ready() {
        let mut reserved = HashSet::new();
        let file = file_record("IMG_1001", "jpg");

        let plan = create_plan_for_file(Path::new(r"C:\todo"), &file, None, &mut reserved).unwrap();

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
        let destination_dir = temp_dir.path().join("2026").join("3").join("5");
        assert!(
            !destination_dir.exists(),
            "planner dry-run must not create filesystem directories"
        );
    }
}
