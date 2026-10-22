//! Picks the places that best name a GPS point.

use super::area::{covering_city, park_or_city};
use super::distance::Coordinates;
use super::index::{Entry, GeoIndex, INDEX};
use super::kind::PlaceKind;
use super::nearby::Nearby;
use super::notable::notable_in;
use super::parks::PARKS;
use super::place::Place;
use super::radius::FALLBACK_RADIUS_KM;

/// Every name a point can get, from one search of its surroundings.
pub(crate) struct Resolution {
    /// What a trip folder is named after: see `nearest_place`.
    pub(crate) place: Option<Place>,
    /// See `nearest_city`.
    pub(crate) city: Option<Place>,
    /// What an outing is named after: see `notable_place`.
    pub(crate) notable: Option<Place>,
}

pub(crate) fn resolve(point: Coordinates) -> Resolution {
    let nearby = Nearby::new(&INDEX, point);
    Resolution {
        place: place_in(&nearby),
        city: city_in(&nearby),
        notable: notable_in(&nearby),
    }
}

/// The park or city that covers `point`, else the point of interest that
/// contains it, else the nearest city within 50 km, else the nearest place of
/// any kind within 150 km.
pub(crate) fn nearest_place(point: Coordinates) -> Option<Place> {
    place_in(&Nearby::new(&INDEX, point))
}

/// The city that covers `point`, else the nearest city within 50 km.
pub(crate) fn nearest_city(point: Coordinates) -> Option<Place> {
    city_in(&Nearby::new(&INDEX, point))
}

/// Country code and English name of the nearest place. Near a border this
/// can name the neighbour, because GeoNames gives points and no outlines.
// Trips read the country off `nearest_place`, which they already call.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn country_of(point: Coordinates) -> Option<(String, String)> {
    nearest_place(point).map(|place| (place.country_code, place.country_name))
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

fn place_in(nearby: &Nearby) -> Option<Place> {
    let index = nearby.index;
    if let Some(place) = park_or_city(nearby, &PARKS).and_then(|area| area.place(index)) {
        return Some(place);
    }
    let entry = nearby
        .matching_poi(|_| true)
        .or_else(|| nearby.nearest_city())
        .or_else(|| nearest(index, nearby.point, FALLBACK_RADIUS_KM, |_| true).map(|(e, _)| e))?;
    Some(index.place(entry))
}

fn city_in(nearby: &Nearby) -> Option<Place> {
    let entry = covering_city(nearby).or_else(|| nearby.nearest_city())?;
    Some(nearby.index.place(entry))
}
