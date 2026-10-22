//! Picks the point of interest an outing is named after.

use super::area::park_place;
use super::distance::Coordinates;
use super::index::INDEX;
use super::kind::PlaceKind;
use super::nearby::Nearby;
use super::parks::PARKS;
use super::place::Place;

/// A visitor attraction that contains `point`, else the park around it, else
/// any other point of interest. A beach inside a park names the park, and a
/// zoo inside a city still names the zoo. It doesn't return a city.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn notable_place(point: Coordinates) -> Option<Place> {
    notable_in(&Nearby::new(&INDEX, point))
}

pub(super) fn notable_in(nearby: &Nearby) -> Option<Place> {
    let index = nearby.index;
    if let Some(attraction) = nearby.matching_poi(is_attraction) {
        return Some(index.place(attraction));
    }
    let park = PARKS.containing(nearby.point);
    if let Some(park) = park.and_then(|park| park_place(index, park)) {
        return Some(park);
    }
    nearby
        .matching_poi(|_| true)
        .map(|entry| index.place(entry))
}

fn is_attraction(kind: PlaceKind) -> bool {
    use PlaceKind::*;
    matches!(
        kind,
        Garden
            | Zoo
            | AmusementPark
            | Museum
            | Castle
            | Palace
            | Monument
            | HistoricSite
            | Ruin
            | Temple
    )
}
