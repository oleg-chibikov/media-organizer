//! Splits geotagged files far from home into trips.

use super::{
    draft::Draft,
    place_table::PlaceTable,
    shots::Shot,
    thresholds::{MIN_GROUP_FILES, TRIP_MAX_GAP, TRIP_MIN_DISTANCE_KM},
    trip_places::group_trip_places,
};
use crate::geo::{distance_km, Coordinates};

/// Runs of away files with at most `TRIP_MAX_GAP` between neighbours.
/// Runs with too few files are dropped. `shots` must be in time order.
pub(super) fn find_trips(shots: &[Shot], home: Coordinates, places: &PlaceTable) -> Vec<Draft> {
    let mut runs: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    for (index, shot) in shots.iter().enumerate() {
        let Some(location) = &shot.location else {
            continue;
        };
        if distance_km(location.point, home) <= TRIP_MIN_DISTANCE_KM {
            continue;
        }
        let continues = current
            .last()
            .is_some_and(|&last| shot.taken - shots[last].taken <= TRIP_MAX_GAP);
        if !continues && !current.is_empty() {
            runs.push(std::mem::take(&mut current));
        }
        current.push(index);
    }
    runs.push(current);
    runs.into_iter()
        .filter(|run| run.len() >= MIN_GROUP_FILES)
        .map(|run| group_trip_places(&run, shots, places))
        .collect()
}
