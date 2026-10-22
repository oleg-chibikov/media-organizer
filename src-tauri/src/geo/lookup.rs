//! Picks the place that best names a GPS point.

use super::area::{covering_city, park_or_city};
use super::distance::Coordinates;
use super::index::{Entry, GeoIndex, INDEX};
use super::kind::PlaceKind;
use super::place::Place;
use super::radius::{CITY_RADIUS_KM, FALLBACK_RADIUS_KM, MAX_POI_RADIUS_KM};

/// The national park or city that covers `point`, else the point of interest
/// that contains it, else the nearest city within 50 km, else the nearest
/// place of any kind within 150 km.
pub(crate) fn nearest_place(point: Coordinates) -> Option<Place> {
    let index = &*INDEX;
    let entry = park_or_city(index, point)
        .or_else(|| matching_poi(index, point, |_| true))
        .or_else(|| nearest_city_entry(index, point))
        .or_else(|| nearest(index, point, FALLBACK_RADIUS_KM, |_| true).map(|(entry, _)| entry))?;
    Some(index.place(entry))
}

/// The city that covers `point`, else the nearest city within 50 km.
pub(crate) fn nearest_city(point: Coordinates) -> Option<Place> {
    let index = &*INDEX;
    let entry = covering_city(index, point)
        .map(|(entry, _)| entry)
        .or_else(|| nearest_city_entry(index, point))?;
    Some(index.place(entry))
}

/// Country code and English name of the nearest place. Near a border this
/// can name the neighbour, because GeoNames gives points and no outlines.
// Trips read the country off `nearest_place`, which they already call.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn country_of(point: Coordinates) -> Option<(String, String)> {
    nearest_place(point).map(|place| (place.country_code, place.country_name))
}

/// The most nested accepted place whose radius covers `point` wins, so a garden
/// beats the park around it. On a tie, the closest relative to its radius wins.
pub(super) fn matching_poi(
    index: &GeoIndex,
    point: Coordinates,
    accepts: impl Fn(PlaceKind) -> bool,
) -> Option<&Entry> {
    index
        .near(point, MAX_POI_RADIUS_KM)
        .filter(|(entry, _)| accepts(entry.kind))
        .filter_map(|(entry, distance)| {
            let radius = entry.kind.match_radius_km()?;
            (distance <= radius).then_some((entry, entry.kind.nesting_level(), distance / radius))
        })
        .min_by(|a, b| a.1.cmp(&b.1).then(a.2.total_cmp(&b.2)))
        .map(|(entry, _, _)| entry)
}

/// The nearest accepted place within `radius_km`, with its distance.
pub(super) fn nearest(
    index: &GeoIndex,
    point: Coordinates,
    radius_km: f64,
    accepts: impl Fn(PlaceKind) -> bool,
) -> Option<(&Entry, f64)> {
    index
        .near(point, radius_km)
        .filter(|(entry, _)| accepts(entry.kind))
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

fn nearest_city_entry(index: &GeoIndex, point: Coordinates) -> Option<&Entry> {
    nearest(index, point, CITY_RADIUS_KM, |kind| kind == PlaceKind::City).map(|(entry, _)| entry)
}
