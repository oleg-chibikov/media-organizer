//! Media files reduced to what grouping needs, in time order.

use std::collections::HashMap;

use chrono::NaiveDateTime;

use super::place_table::{PlaceTable, Resolved};
use crate::{
    geo::Coordinates,
    models::{FileKind, FileRecord, MetadataResult, MetadataStatus},
};

/// The format of `MetadataResult::chosen_date`.
pub(super) const CHOSEN_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub(super) struct Shot<'a> {
    pub(super) file_id: &'a str,
    pub(super) taken: NaiveDateTime,
    pub(super) location: Option<Location>,
}

#[derive(Clone, Copy)]
pub(super) struct Location {
    pub(super) point: Coordinates,
    pub(super) resolved: Resolved,
}

/// Photos and videos with a chosen date, sorted by date. Geocodes each GPS point.
pub(super) fn collect_shots<'a>(
    files: &'a [FileRecord],
    metadata_results: &[MetadataResult],
    places: &mut PlaceTable,
) -> Vec<Shot<'a>> {
    let metadata_by_id: HashMap<&str, &MetadataResult> = metadata_results
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
            let location = metadata.gps.map(|gps| {
                let point = Coordinates::from(gps);
                Location {
                    point,
                    resolved: places.resolve(point),
                }
            });
            Some(Shot {
                file_id: file.id.as_str(),
                taken,
                location,
            })
        })
        .collect();
    shots.sort_by(|a, b| a.taken.cmp(&b.taken).then(a.file_id.cmp(b.file_id)));
    shots
}
