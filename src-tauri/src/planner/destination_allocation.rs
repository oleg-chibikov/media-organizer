use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use super::content_match::files_have_identical_contents;
use crate::models::{ConflictStatus, PlanStatus};

/// Picks the first free `<stem>[ (n)].<ext>` path, blocking exact duplicates already on disk.
pub(super) fn allocate_destination_path(
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

        if exists_on_disk
            && suffix == 0
            && files_have_identical_contents(source_path, &destination_path)?
        {
            return Ok((
                destination_path,
                filename,
                ConflictStatus::Blocked,
                PlanStatus::BlockedConflict,
            ));
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

fn normalize_path_key(path: &Path) -> String {
    path.to_string_lossy().to_ascii_lowercase()
}
