//! Places files with no GPS next to geotagged files of the same device.

use std::collections::HashMap;

use chrono::NaiveDateTime;

use super::{shots::Shot, thresholds::UNTAGGED_MAX_GAP};
use crate::models::FilePlace;

/// A file with no GPS takes the place of the closest geotagged file from the same device
/// within 3 hours. A file with no device name, or none close, stays out.
pub(super) fn untagged_places(
    shots: &[Shot],
    tagged: &HashMap<String, FilePlace>,
) -> Vec<(String, FilePlace)> {
    // Shots are in time order, so each device's list is too.
    let mut by_device: HashMap<&str, Vec<(NaiveDateTime, &FilePlace)>> = HashMap::new();
    for shot in shots {
        if let (Some(device), Some(place)) = (shot.device, tagged.get(shot.file_id)) {
            by_device
                .entry(device)
                .or_default()
                .push((shot.taken, place));
        }
    }
    shots
        .iter()
        .filter(|shot| shot.location.is_none())
        .filter_map(|shot| {
            let anchors = by_device.get(shot.device?)?;
            let (taken, place) = nearest(anchors, shot.taken)?;
            ((taken - shot.taken).abs() <= UNTAGGED_MAX_GAP)
                .then(|| (shot.file_id.to_owned(), place.clone()))
        })
        .collect()
}

/// The anchor closest to `taken` in a list sorted by time.
fn nearest<'a>(
    anchors: &[(NaiveDateTime, &'a FilePlace)],
    taken: NaiveDateTime,
) -> Option<(NaiveDateTime, &'a FilePlace)> {
    let after = anchors.partition_point(|(time, _)| *time <= taken);
    let before = after.checked_sub(1).map(|index| anchors[index]);
    before
        .into_iter()
        .chain(anchors.get(after).copied())
        .min_by_key(|(time, _)| (*time - taken).abs())
}
