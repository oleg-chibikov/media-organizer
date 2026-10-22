use std::{collections::HashSet, path::Path};

use chrono::NaiveDateTime;

use super::{destination_allocation::allocate_destination_path, DestinationLayout, PlaceFolders};
use crate::{
    file_places::CHOSEN_DATE_FORMAT,
    filename,
    models::{
        ConflictStatus, FileRecord, MetadataResult, MetadataStatus, MovePlan, PlanStatus,
        ScanStatus,
    },
};

/// Plans `<folder>/<date>[ <place>, <country>] (<original>)[ - <device>].<ext>` for one file.
/// A file with a place goes to its place folder.
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
    let place_dir = place_folders.relative_dir(&file.id, &parsed_date);
    let destination_dir = layout.destination_dir(source_path, &parsed_date, place_dir)?;
    let original_name = filename::sanitize_for_windows_filename(&file.original_name);
    let device_name = metadata.and_then(|metadata| metadata.device_name.as_deref());
    let place_label = place_folders.label(&file.id);
    let stem_base = destination_stem(&parsed_date, place_label, &original_name, device_name);

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
        place: place_folders.place(&file.id).cloned(),
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

/// The place sits before the original name, so a place with brackets such as
/// `Central Park (New York)` doesn't hide the original name from `parse_original_name`.
fn destination_stem(
    date: &NaiveDateTime,
    place_label: Option<&str>,
    original_name: &str,
    device_name: Option<&str>,
) -> String {
    let mut stem_base = date.format("%Y-%m-%d %H-%M-%S").to_string();
    if let Some(place_label) = place_label {
        stem_base.push(' ');
        stem_base.push_str(place_label);
    }
    stem_base.push_str(&format!(" ({original_name})"));
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
        place: None,
    }
}
