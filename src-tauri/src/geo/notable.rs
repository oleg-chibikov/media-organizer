//! Picks the point of interest an outing is named after.

use super::area::park_or_city;
use super::distance::Coordinates;
use super::index::INDEX;
use super::kind::PlaceKind;
use super::lookup::matching_poi;
use super::place::Place;

/// A visitor attraction that contains `point`, else the national park around
/// it, else any other point of interest. A beach inside a park names the park,
/// and a zoo inside a city still names the zoo. It doesn't return a city.
pub(crate) fn notable_place(point: Coordinates) -> Option<Place> {
    let index = &*INDEX;
    let park = || park_or_city(index, point).filter(|entry| entry.kind != PlaceKind::City);
    let entry = matching_poi(index, point, is_attraction)
        .or_else(park)
        .or_else(|| matching_poi(index, point, |_| true))?;
    Some(index.place(entry))
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
