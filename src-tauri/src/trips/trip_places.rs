//! Splits one trip into place folders, with small places folded into bigger ones.

use std::collections::HashMap;

use super::{
    draft::{Anchor, Draft, DraftGroup},
    place_table::{PlaceId, PlaceTable},
    shots::Shot,
    thresholds::MIN_GROUP_FILES,
};
use crate::{geo::PlaceKind, models::SuggestionKind};

/// Groups `run` by resolved place across the whole trip. A small point of
/// interest falls back to its city, and a small city to the trip root.
pub(super) fn group_trip_places(run: &[usize], shots: &[Shot], places: &PlaceTable) -> Draft {
    let resolved: Vec<_> = run
        .iter()
        .map(|&index| shots[index].location.map(|location| location.resolved))
        .collect();
    let place_counts = count(resolved.iter().map(|r| r.and_then(|r| r.place)));
    let targets: Vec<Option<PlaceId>> = resolved
        .iter()
        .map(|resolved| {
            let resolved = (*resolved)?;
            match resolved.place {
                Some(place) if places.get(place).kind == PlaceKind::City => Some(place),
                Some(place) if place_counts[&place] >= MIN_GROUP_FILES => Some(place),
                _ => resolved.city,
            }
        })
        .collect();
    let target_counts = count(targets.iter().copied());

    let mut draft = Draft {
        kind: SuggestionKind::Trip,
        groups: Vec::new(),
        root: Vec::new(),
        anchors: Vec::with_capacity(run.len()),
    };
    let mut group_of: HashMap<PlaceId, usize> = HashMap::new();
    for (&index, target) in run.iter().zip(targets) {
        let group = target
            .filter(|place| target_counts[place] >= MIN_GROUP_FILES)
            .map(|place| {
                *group_of.entry(place).or_insert_with(|| {
                    draft.groups.push(DraftGroup {
                        place,
                        shots: Vec::new(),
                    });
                    draft.groups.len() - 1
                })
            });
        draft.add(index, group);
        draft.anchors.push(Anchor {
            taken: shots[index].taken,
            group,
        });
    }
    draft
}

fn count(places: impl Iterator<Item = Option<PlaceId>>) -> HashMap<PlaceId, usize> {
    let mut counts = HashMap::new();
    for place in places.flatten() {
        *counts.entry(place).or_insert(0) += 1;
    }
    counts
}
