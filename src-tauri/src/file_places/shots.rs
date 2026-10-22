//! Media files reduced to what place detection needs, in time order.

use std::collections::HashMap;

use chrono::NaiveDateTime;

use super::place_table::{PlaceTable, Resolved};
use crate::{
    geo::Coordinates,
    models::{FileKind, FileRecord, MetadataResult, MetadataStatus},
};

/// The format of `MetadataResult::chosen_date`.
pub(crate) const CHOSEN_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub(super) struct Shot<'a> {
    pub(super) file_id: &'a str,
    pub(super) taken: NaiveDateTime,
    /// The camera or phone, so a file with no GPS joins a place of the same device.
    pub(super) device: Option<&'a str>,
    /// The place and city of the file's GPS point.
    pub(super) location: Option<Resolved>,
}

/// Photos and videos with a chosen date, sorted by date. Geocodes each GPS point.
pub(super) fn collect_shots<'a>(
    files: &'a [FileRecord],
    metadata_results: &'a [MetadataResult],
    places: &mut PlaceTable,
) -> Vec<Shot<'a>> {
    let metadata_by_id: HashMap<&str, &'a MetadataResult> = metadata_results
        .iter()
        .map(|result| (result.file_id.as_str(), result))
        .collect();
    let mut shots: Vec<Shot> = files
        .iter()
        .filter(|file| matches!(file.kind, FileKind::Photo | FileKind::Video))
        .filter_map(|file| {
            let metadata = metadata_by_id.get(file.id.as_str())?;
            if !matches!(metadata.metadata_status, MetadataStatus::Ready) {
                return None;
            }
            let date = metadata.chosen_date.as_deref()?;
            let taken = NaiveDateTime::parse_from_str(date, CHOSEN_DATE_FORMAT).ok()?;
            let location = metadata
                .gps
                .map(|gps| places.resolve(Coordinates::from(gps)));
            Some(Shot {
                file_id: file.id.as_str(),
                taken,
                device: metadata
                    .device_name
                    .as_deref()
                    .filter(|name| !name.is_empty()),
                location,
            })
        })
        .collect();
    shots.sort_by(|a, b| a.taken.cmp(&b.taken).then(a.file_id.cmp(b.file_id)));
    shots
}
