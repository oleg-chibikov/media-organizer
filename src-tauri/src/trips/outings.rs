//! Finds visits to notable places outside trips.

use std::collections::HashMap;

use chrono::Datelike;

use super::{
    draft::{Anchor, Draft, DraftGroup},
    place_table::PlaceId,
    shots::Shot,
    thresholds::MIN_GROUP_FILES,
};
use crate::models::SuggestionKind;

/// One outing per point of interest and year, for geotagged shots outside trips.
/// Named after `Resolved::notable`, so cities make no outing. `shots` must be in time order.
pub(super) fn find_outings(shots: &[Shot], in_trip: &[bool]) -> Vec<Draft> {
    let mut drafts: Vec<Draft> = Vec::new();
    let mut draft_of: HashMap<(PlaceId, i32), usize> = HashMap::new();
    for (index, shot) in shots.iter().enumerate() {
        let place = shot.location.and_then(|location| location.resolved.notable);
        let Some(place) = place.filter(|_| !in_trip[index]) else {
            continue;
        };
        let draft = *draft_of
            .entry((place, shot.taken.year()))
            .or_insert_with(|| {
                drafts.push(new_outing(place));
                drafts.len() - 1
            });
        drafts[draft].add(index, Some(0));
        drafts[draft].anchors.push(Anchor {
            taken: shot.taken,
            group: Some(0),
        });
    }
    drafts.retain(|draft| draft.anchors.len() >= MIN_GROUP_FILES);
    drafts
}

fn new_outing(place: PlaceId) -> Draft {
    Draft {
        kind: SuggestionKind::Outing,
        groups: vec![DraftGroup {
            place,
            shots: Vec::new(),
        }],
        root: Vec::new(),
        anchors: Vec::new(),
    }
}
