use std::{collections::HashSet, path::Path};

use chrono::NaiveDateTime;

use super::{destination_allocation::allocate_destination_path, DestinationLayout, PlaceFolders};
use crate::{
    filename,
    models::{
        ConflictStatus, FileRecord, MetadataResult, MetadataStatus, MovePlan, PlanStatus,
        ScanStatus,
    },
    trips::CHOSEN_DATE_FORMAT,
};

/// Plans `<folder>/<date> (<original>) - <device>.<ext>` for one file. A file in an accepted
/// place folder goes there, and a file earlier runs placed stays in its folder.
pub(super) fn create_plan_for_file(
    layout: &DestinationLayout,
    file: &FileRecord,
    metadata: Option<&MetadataResult>,
    place_folders: &PlaceFolders,
    reserved_paths: &mut HashSet<String>,
) -> Result<MovePlan, String> {
    let Some(chosen_date) = plannable_date(file, metadata) else {
        return Ok(empty_plan(file, PlanStatus::BlockedNoDate));
    };

    let parsed_date = NaiveDateTime::parse_from_str(chosen_date, CHOSEN_DATE_FORMAT)
        .map_err(|error| format!("Invalid chosen date for {}: {error}", file.source_path))?;

    let source_path = Path::new(&file.source_path);
    let place = place_folders.get(&file.id);
    let placed = place
        .is_none()
        .then(|| place_folders.placed(source_path))
        .flatten();
    let destination_dir = match (placed, source_path.parent()) {
        (Some(_), Some(folder)) => folder.to_path_buf(),
        _ => layout.destination_dir(source_path, &parsed_date, place)?,
    };
    let original_name = filename::sanitize_for_windows_filename(&file.original_name);
    let device_name = metadata.and_then(|metadata| metadata.device_name.as_deref());
    let stem_base = destination_stem(&parsed_date, &original_name, device_name);

    let (destination_path, destination_filename, conflict_status, plan_status) =
        allocate_destination_path(
            source_path,
            &destination_dir,
            &stem_base,
            &file.extension,
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
        placement: place
            .map(|place| place.placement.clone())
            .or_else(|| placed.cloned()),
    })
}

fn plannable_date<'a>(file: &FileRecord, metadata: Option<&'a MetadataResult>) -> Option<&'a str> {
    if !matches!(file.scan_status, ScanStatus::Discovered) {
        return None;
    }
    let metadata = metadata?;
    if !matches!(metadata.metadata_status, MetadataStatus::Ready) {
        return None;
    }
    metadata.chosen_date.as_deref()
}

fn destination_stem(
    date: &NaiveDateTime,
    original_name: &str,
    device_name: Option<&str>,
) -> String {
    let mut stem_base = format!("{} ({original_name})", date.format("%Y-%m-%d %H-%M-%S"));
    if let Some(device_name) = device_name {
        let sanitized_device_name = filename::sanitize_for_windows_filename(device_name);
        if !sanitized_device_name.is_empty() {
            stem_base.push_str(" - ");
            stem_base.push_str(&sanitized_device_name);
        }
    }
    stem_base
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
        placement: None,
    }
}
