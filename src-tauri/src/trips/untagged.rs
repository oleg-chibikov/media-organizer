//! Places files with no GPS next to geotagged files taken around the same time.

use chrono::NaiveDateTime;

use super::{
    draft::Draft,
    shots::Shot,
    thresholds::{OUTING_UNTAGGED_MAX_GAP, TRIP_UNTAGGED_MAX_GAP},
};
use crate::models::SuggestionKind;

/// Inside a trip's time range a file joins the place of the nearest trip
/// file within 3 hours, else the trip root. Outside trips it joins an outing
/// with a file within 1 hour on the same day. Other files stay out.
pub(super) fn attach_untagged(shots: &[Shot], drafts: &mut [Draft]) {
    let mut trips: Vec<usize> = (0..drafts.len())
        .filter(|&draft| drafts[draft].kind == SuggestionKind::Trip)
        .collect();
    trips.sort_by_key(|&draft| drafts[draft].first_taken());
    let mut outing_anchors: Vec<(NaiveDateTime, usize)> = (0..drafts.len())
        .filter(|&draft| drafts[draft].kind == SuggestionKind::Outing)
        .flat_map(|draft| drafts[draft].anchors.iter().map(move |a| (a.taken, draft)))
        .collect();
    outing_anchors.sort_unstable();

    for (index, shot) in shots.iter().enumerate() {
        if shot.location.is_some() {
            continue;
        }
        let taken = shot.taken;
        if let Some(trip) = trip_covering(&trips, drafts, taken) {
            let draft = &mut drafts[trip];
            let group = nearest(&draft.anchors, taken, |anchor| anchor.taken)
                .filter(|anchor| (anchor.taken - taken).abs() <= TRIP_UNTAGGED_MAX_GAP)
                .and_then(|anchor| anchor.group);
            draft.add(index, group);
            continue;
        }
        let outing = nearest(&outing_anchors, taken, |anchor| anchor.0).filter(|(near, _)| {
            near.date() == taken.date() && (*near - taken).abs() <= OUTING_UNTAGGED_MAX_GAP
        });
        if let Some(&(_, draft)) = outing {
            drafts[draft].add(index, Some(0));
        }
    }
}

fn trip_covering(trips: &[usize], drafts: &[Draft], taken: NaiveDateTime) -> Option<usize> {
    let after = trips.partition_point(|&draft| drafts[draft].first_taken() <= taken);
    let trip = trips[..after].last().copied()?;
    (drafts[trip].last_taken() >= taken).then_some(trip)
}

/// The item closest to `taken` in a slice sorted by time.
fn nearest<T>(items: &[T], taken: NaiveDateTime, time: impl Fn(&T) -> NaiveDateTime) -> Option<&T> {
    let after = items.partition_point(|item| time(item) <= taken);
    let before = after.checked_sub(1).map(|index| &items[index]);
    let distance = |item: &&T| (time(item) - taken).abs();
    before
        .into_iter()
        .chain(items.get(after))
        .min_by_key(distance)
}
