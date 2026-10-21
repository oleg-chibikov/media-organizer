use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::{
    file_ops::files_have_same_bytes,
    models::{ConflictStatus, PlanStatus},
    path_safety::normalized_path_string,
};

/// Picks the first free `<stem>[ (n)].<ext>` path. An earlier copy of the same bytes under
/// any of those names blocks the plan, so a kept source isn't copied again on the next run.
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
        let path_key = normalized_path_string(&destination_path);

        let exists_on_disk = destination_path.exists();
        let exists_in_run = reserved_paths.contains(&path_key);

        if exists_on_disk
            && files_have_same_bytes(source_path, &destination_path)
                .map_err(|error| format!("Failed to compare files for conflict check: {error}"))?
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
