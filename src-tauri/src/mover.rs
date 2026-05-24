use std::{
    collections::{HashMap, HashSet},
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
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

#[derive(Debug, Clone, Default)]
pub struct SafetyCheckedMoves {
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

pub fn validate_move_safety(
    selected_root: &Path,
    operations: Vec<MoveOperation>,
) -> SafetyCheckedMoves {
    let processed_root = selected_root.join("Processed");
    validate_move_safety_internal(
        selected_root,
        &processed_root,
        operations,
        "Source file is inside the Processed folder.",
        "Destination file is outside the selected Processed folder.",
    )
}

pub fn validate_move_safety_with_roots(
    selected_source_root: &Path,
    effective_destination_root: &Path,
    operations: Vec<MoveOperation>,
) -> SafetyCheckedMoves {
    validate_move_safety_internal(
        selected_source_root,
        effective_destination_root,
        operations,
        "Source file is inside the effective destination root.",
        "Destination file is outside the effective destination root.",
    )
}

fn validate_move_safety_internal(
    selected_source_root: &Path,
    effective_destination_root: &Path,
    operations: Vec<MoveOperation>,
    source_inside_destination_error: &str,
    destination_outside_root_error: &str,
) -> SafetyCheckedMoves {
    let mut ready = Vec::new();
    let mut skipped = Vec::new();

    for operation in operations {
        let source = Path::new(&operation.source_path);
        let destination = Path::new(&operation.destination_path);

        if has_parent_dir_component(source) || has_parent_dir_component(destination) {
            skipped.push(skipped_result(
                &operation.file_id,
                &operation.source_path,
                &operation.destination_path,
                "Source and destination paths must not contain parent-directory traversal.",
            ));
            continue;
        }

        if paths_equal_case_insensitive(source, destination) {
            skipped.push(skipped_result(
                &operation.file_id,
                &operation.source_path,
                &operation.destination_path,
                "Source and destination resolve to the same path.",
            ));
            continue;
        }

        if !path_is_within_case_insensitive(source, selected_source_root) {
            skipped.push(skipped_result(
                &operation.file_id,
                &operation.source_path,
                &operation.destination_path,
                "Source file is outside the selected folder.",
            ));
            continue;
        }

        if path_is_within_case_insensitive(source, effective_destination_root) {
            skipped.push(skipped_result(
                &operation.file_id,
                &operation.source_path,
                &operation.destination_path,
                source_inside_destination_error,
            ));
            continue;
        }

        if !path_is_within_case_insensitive(destination, effective_destination_root) {
            skipped.push(skipped_result(
                &operation.file_id,
                &operation.source_path,
                &operation.destination_path,
                destination_outside_root_error,
            ));
            continue;
        }

        if !source.is_file() {
            skipped.push(skipped_result(
                &operation.file_id,
                &operation.source_path,
                &operation.destination_path,
                "Source path is not a regular file at execution time.",
            ));
            continue;
        }

        ready.push(operation);
    }

    SafetyCheckedMoves { ready, skipped }
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

fn has_parent_dir_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

fn path_is_within_case_insensitive(candidate: &Path, parent: &Path) -> bool {
    let candidate = normalized_path_string(candidate);
    let parent = normalized_path_string(parent);

    candidate == parent || candidate.starts_with(&format!("{parent}/"))
}

fn paths_equal_case_insensitive(left: &Path, right: &Path) -> bool {
    normalized_path_string(left) == normalized_path_string(right)
}

fn normalized_path_string(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_ascii_lowercase()
}

pub fn execute_move_with_rename(operation: &MoveOperation) -> MoveResult {
    execute_move_with_rename_internal(operation, |source, destination| {
        fs::rename(source, destination)
    })
}

fn execute_move_with_rename_internal<F>(operation: &MoveOperation, rename_fn: F) -> MoveResult
where
    F: Fn(&Path, &Path) -> io::Result<()>,
{
    let source = Path::new(&operation.source_path);
    let destination = Path::new(&operation.destination_path);

    if destination.exists() {
        return skipped_result(
            &operation.file_id,
            &operation.source_path,
            &operation.destination_path,
            "Destination already exists. Refusing to overwrite.",
        );
    }

    if !source.exists() {
        return MoveResult {
            file_id: operation.file_id.clone(),
            attempted_source_path: operation.source_path.clone(),
            attempted_destination_path: operation.destination_path.clone(),
            move_strategy: MoveStrategy::Rename,
            result_status: MoveResultStatus::Failed,
            error: Some("Source file no longer exists at execution time.".to_string()),
        };
    }

    if let Some(parent) = destination.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            return MoveResult {
                file_id: operation.file_id.clone(),
                attempted_source_path: operation.source_path.clone(),
                attempted_destination_path: operation.destination_path.clone(),
                move_strategy: MoveStrategy::Rename,
                result_status: MoveResultStatus::Failed,
                error: Some(format!(
                    "Failed to create destination directory before rename: {error}"
                )),
            };
        }
    }

    match rename_fn(source, destination) {
        Ok(()) => MoveResult {
            file_id: operation.file_id.clone(),
            attempted_source_path: operation.source_path.clone(),
            attempted_destination_path: operation.destination_path.clone(),
            move_strategy: MoveStrategy::Rename,
            result_status: MoveResultStatus::Moved,
            error: None,
        },
        Err(error) => {
            if should_fallback_to_copy_keep_source(&error) {
                return execute_copy_verify_keep_source(operation, source, destination);
            }

            MoveResult {
                file_id: operation.file_id.clone(),
                attempted_source_path: operation.source_path.clone(),
                attempted_destination_path: operation.destination_path.clone(),
                move_strategy: MoveStrategy::Rename,
                result_status: MoveResultStatus::Failed,
                error: Some(format!("Atomic rename failed: {error}")),
            }
        }
    }
}

fn should_fallback_to_copy_keep_source(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::CrossesDevices || error.raw_os_error() == Some(17)
}

fn execute_copy_verify_keep_source(
    operation: &MoveOperation,
    source: &Path,
    destination: &Path,
) -> MoveResult {
    if destination.exists() {
        return skipped_result(
            &operation.file_id,
            &operation.source_path,
            &operation.destination_path,
            "Destination already exists. Refusing to overwrite.",
        );
    }

    let (temp_destination, mut temp_file) = match reserve_temp_destination(destination) {
        Ok(reserved) => reserved,
        Err(error) => {
            return MoveResult {
                file_id: operation.file_id.clone(),
                attempted_source_path: operation.source_path.clone(),
                attempted_destination_path: operation.destination_path.clone(),
                move_strategy: MoveStrategy::CopyKeepSource,
                result_status: MoveResultStatus::Failed,
                error: Some(format!("Failed to reserve temporary copy path: {error}")),
            };
        }
    };

    if let Err(error) = copy_without_overwrite(source, &mut temp_file) {
        drop(temp_file);
        let _ = fs::remove_file(&temp_destination);
        return MoveResult {
            file_id: operation.file_id.clone(),
            attempted_source_path: operation.source_path.clone(),
            attempted_destination_path: operation.destination_path.clone(),
            move_strategy: MoveStrategy::CopyKeepSource,
            result_status: MoveResultStatus::Failed,
            error: Some(format!("Copy fallback failed: {error}")),
        };
    }

    drop(temp_file);

    if let Err(error) = verify_destination_matches_source(source, &temp_destination) {
        let _ = fs::remove_file(&temp_destination);
        return MoveResult {
            file_id: operation.file_id.clone(),
            attempted_source_path: operation.source_path.clone(),
            attempted_destination_path: operation.destination_path.clone(),
            move_strategy: MoveStrategy::CopyKeepSource,
            result_status: MoveResultStatus::Failed,
            error: Some(error),
        };
    }

    if destination.exists() {
        let _ = fs::remove_file(&temp_destination);
        return skipped_result(
            &operation.file_id,
            &operation.source_path,
            &operation.destination_path,
            "Destination appeared during copy fallback. Refusing to overwrite.",
        );
    }

    if let Err(error) = persist_verified_copy(&temp_destination, destination) {
        let _ = fs::remove_file(&temp_destination);
        return MoveResult {
            file_id: operation.file_id.clone(),
            attempted_source_path: operation.source_path.clone(),
            attempted_destination_path: operation.destination_path.clone(),
            move_strategy: MoveStrategy::CopyKeepSource,
            result_status: MoveResultStatus::Failed,
            error: Some(format!("Failed to persist verified copy fallback: {error}")),
        };
    }

    let _ = fs::remove_file(&temp_destination);

    if let Err(error) = verify_destination_matches_source(source, destination) {
        return MoveResult {
            file_id: operation.file_id.clone(),
            attempted_source_path: operation.source_path.clone(),
            attempted_destination_path: operation.destination_path.clone(),
            move_strategy: MoveStrategy::CopyKeepSource,
            result_status: MoveResultStatus::Failed,
            error: Some(format!(
                "{error} Source file was kept; destination copy should be inspected."
            )),
        };
    }

    MoveResult {
        file_id: operation.file_id.clone(),
        attempted_source_path: operation.source_path.clone(),
        attempted_destination_path: operation.destination_path.clone(),
        move_strategy: MoveStrategy::CopyKeepSource,
        result_status: MoveResultStatus::Copied,
        error: None,
    }
}

fn verify_destination_matches_source(source: &Path, destination: &Path) -> Result<(), String> {
    if !destination.exists() {
        return Err("Destination copy is missing after copy step.".to_string());
    }

    let source_size = fs::metadata(source)
        .map_err(|error| format!("Failed to read source metadata for verification: {error}"))?
        .len();
    let destination_size = fs::metadata(destination)
        .map_err(|error| format!("Failed to read destination metadata for verification: {error}"))?
        .len();

    if source_size != destination_size {
        return Err(format!(
            "Destination copy size mismatch: source={source_size} bytes, destination={destination_size} bytes."
        ));
    }

    let mut source_file = File::open(source)
        .map_err(|error| format!("Failed to open source for byte verification: {error}"))?;
    let mut destination_file = File::open(destination)
        .map_err(|error| format!("Failed to open destination for byte verification: {error}"))?;
    let mut source_buffer = [0_u8; 64 * 1024];
    let mut destination_buffer = [0_u8; 64 * 1024];

    loop {
        let source_read = source_file
            .read(&mut source_buffer)
            .map_err(|error| format!("Failed to read source for byte verification: {error}"))?;
        let destination_read = destination_file
            .read(&mut destination_buffer)
            .map_err(|error| {
                format!("Failed to read destination for byte verification: {error}")
            })?;

        if source_read != destination_read
            || source_buffer[..source_read] != destination_buffer[..destination_read]
        {
            return Err("Destination copy content mismatch.".to_string());
        }

        if source_read == 0 {
            break;
        }
    }

    Ok(())
}

fn reserve_temp_destination(destination: &Path) -> io::Result<(PathBuf, File)> {
    let parent = destination
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "destination has no parent"))?;
    let file_name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "destination has no file name")
        })?;

    for attempt in 0..1000_u32 {
        let temp_name = format!(
            ".{file_name}.media-organizer-copying-{}-{attempt}.tmp",
            std::process::id()
        );
        let temp_path = parent.join(temp_name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "unable to allocate a unique temporary destination",
    ))
}

fn copy_without_overwrite(source: &Path, destination_file: &mut File) -> io::Result<()> {
    let mut source_file = File::open(source)?;
    io::copy(&mut source_file, destination_file)?;
    destination_file.flush()?;
    destination_file.sync_all()?;
    Ok(())
}

fn persist_verified_copy(temp_destination: &Path, destination: &Path) -> io::Result<()> {
    let mut destination_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    copy_without_overwrite(temp_destination, &mut destination_file)
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
            device_name: None,
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

    #[test]
    fn rename_move_succeeds_and_removes_source() {
        let temp = tempdir().expect("temp dir");
        let source = temp.path().join("source.jpg");
        let destination = temp.path().join("Processed\\2026\\05\\23\\destination.jpg");
        fs::write(&source, b"source").expect("write source");

        let operation = MoveOperation {
            file_id: "photo_1".to_string(),
            source_path: source.to_string_lossy().to_string(),
            destination_path: destination.to_string_lossy().to_string(),
        };

        let result = execute_move_with_rename(&operation);

        assert!(matches!(result.move_strategy, MoveStrategy::Rename));
        assert!(matches!(result.result_status, MoveResultStatus::Moved));
        assert!(!source.exists());
        assert!(destination.exists());
        assert_eq!(fs::read(&destination).expect("read destination"), b"source");
    }

    #[test]
    fn rename_move_refuses_to_overwrite_existing_destination() {
        let temp = tempdir().expect("temp dir");
        let source = temp.path().join("source.jpg");
        let destination = temp.path().join("destination.jpg");
        fs::write(&source, b"source").expect("write source");
        fs::write(&destination, b"existing").expect("write destination");

        let operation = MoveOperation {
            file_id: "photo_1".to_string(),
            source_path: source.to_string_lossy().to_string(),
            destination_path: destination.to_string_lossy().to_string(),
        };

        let result = execute_move_with_rename(&operation);

        assert!(matches!(result.move_strategy, MoveStrategy::NotAttempted));
        assert!(matches!(result.result_status, MoveResultStatus::Skipped));
        assert!(source.exists());
        assert_eq!(
            fs::read(&destination).expect("read destination"),
            b"existing"
        );
    }

    #[test]
    fn falls_back_to_copy_keep_source_when_rename_reports_cross_device() {
        let temp = tempdir().expect("temp dir");
        let source = temp.path().join("source.jpg");
        let destination = temp.path().join("Processed\\2026\\05\\23\\destination.jpg");
        fs::write(&source, b"source").expect("write source");

        let operation = MoveOperation {
            file_id: "photo_1".to_string(),
            source_path: source.to_string_lossy().to_string(),
            destination_path: destination.to_string_lossy().to_string(),
        };

        let result = execute_move_with_rename_internal(&operation, |_src, _dest| {
            Err(io::Error::from_raw_os_error(17))
        });

        assert!(matches!(result.move_strategy, MoveStrategy::CopyKeepSource));
        assert!(matches!(result.result_status, MoveResultStatus::Copied));
        assert!(source.exists());
        assert!(destination.exists());
        assert_eq!(fs::read(&source).expect("read source"), b"source");
        assert_eq!(fs::read(&destination).expect("read destination"), b"source");
    }

    #[test]
    fn verification_fails_when_destination_size_differs() {
        let temp = tempdir().expect("temp dir");
        let source = temp.path().join("source.jpg");
        let destination = temp.path().join("destination.jpg");
        fs::write(&source, b"123456").expect("write source");
        fs::write(&destination, b"123").expect("write destination");

        let verify = verify_destination_matches_source(&source, &destination);

        assert!(verify.is_err());
        let error_message = verify.expect_err("expected verify failure");
        assert!(error_message.contains("size mismatch"));
    }

    #[test]
    fn verification_fails_when_destination_content_differs() {
        let temp = tempdir().expect("temp dir");
        let source = temp.path().join("source.jpg");
        let destination = temp.path().join("destination.jpg");
        fs::write(&source, b"123456").expect("write source");
        fs::write(&destination, b"abcdef").expect("write destination");

        let verify = verify_destination_matches_source(&source, &destination);

        assert!(verify.is_err());
        let error_message = verify.expect_err("expected verify failure");
        assert!(error_message.contains("content mismatch"));
    }

    #[test]
    fn safety_check_rejects_destination_outside_processed_folder() {
        let temp = tempdir().expect("temp dir");
        let source = temp.path().join("source.jpg");
        let destination = temp.path().join("outside.jpg");
        fs::write(&source, b"source").expect("write source");

        let checked = validate_move_safety(
            temp.path(),
            vec![MoveOperation {
                file_id: "photo_1".to_string(),
                source_path: source.to_string_lossy().to_string(),
                destination_path: destination.to_string_lossy().to_string(),
            }],
        );

        assert_eq!(checked.ready.len(), 0);
        assert_eq!(checked.skipped.len(), 1);
        assert!(checked.skipped[0]
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("outside the selected Processed folder"));
        assert!(source.exists());
    }

    #[test]
    fn safety_check_rejects_source_inside_processed_folder() {
        let temp = tempdir().expect("temp dir");
        let source = temp.path().join("Processed").join("source.jpg");
        let destination = temp
            .path()
            .join("Processed")
            .join("2026")
            .join("destination.jpg");
        fs::create_dir_all(source.parent().expect("source parent")).expect("create parent");
        fs::write(&source, b"source").expect("write source");

        let checked = validate_move_safety(
            temp.path(),
            vec![MoveOperation {
                file_id: "photo_1".to_string(),
                source_path: source.to_string_lossy().to_string(),
                destination_path: destination.to_string_lossy().to_string(),
            }],
        );

        assert_eq!(checked.ready.len(), 0);
        assert_eq!(checked.skipped.len(), 1);
        assert!(checked.skipped[0]
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("inside the Processed folder"));
        assert!(source.exists());
    }

    #[test]
    fn safety_check_with_custom_root_rejects_destination_outside_effective_root() {
        let temp = tempdir().expect("temp dir");
        let source_root = temp.path().join("todo");
        let destination_root = temp.path().join("media-root");
        let source = source_root.join("source.jpg");
        let destination = temp.path().join("outside.jpg");
        fs::create_dir_all(&source_root).expect("create source root");
        fs::create_dir_all(&destination_root).expect("create destination root");
        fs::write(&source, b"source").expect("write source");

        let checked = validate_move_safety_with_roots(
            &source_root,
            &destination_root,
            vec![MoveOperation {
                file_id: "photo_1".to_string(),
                source_path: source.to_string_lossy().to_string(),
                destination_path: destination.to_string_lossy().to_string(),
            }],
        );

        assert_eq!(checked.ready.len(), 0);
        assert_eq!(checked.skipped.len(), 1);
        assert!(checked.skipped[0]
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("outside the effective destination root"));
        assert!(source.exists());
    }

    #[test]
    fn safety_check_with_custom_root_rejects_source_inside_effective_root() {
        let temp = tempdir().expect("temp dir");
        let source_root = temp.path().join("todo");
        let destination_root = temp.path().to_path_buf();
        let source = source_root.join("source.jpg");
        let destination = destination_root.join("2026").join("destination.jpg");
        fs::create_dir_all(&source_root).expect("create source root");
        fs::write(&source, b"source").expect("write source");

        let checked = validate_move_safety_with_roots(
            &source_root,
            &destination_root,
            vec![MoveOperation {
                file_id: "photo_1".to_string(),
                source_path: source.to_string_lossy().to_string(),
                destination_path: destination.to_string_lossy().to_string(),
            }],
        );

        assert_eq!(checked.ready.len(), 0);
        assert_eq!(checked.skipped.len(), 1);
        assert!(checked.skipped[0]
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("inside the effective destination root"));
        assert!(source.exists());
    }
}
