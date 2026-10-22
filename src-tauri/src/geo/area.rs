//! Names a point by the broad area it sits in: a national park or a city.

use super::distance::Coordinates;
use super::index::{Entry, GeoIndex};
use super::kind::PlaceKind;
use super::lookup::nearest;
use super::radius::{city_reach_km, CITY_RADIUS_KM};

/// Smaller cities lose to the national park around them, as Katoomba does.
const TOWN_MAX_POPULATION: u32 = 50_000;
/// A big city loses to a park only when its centre is this many times farther
/// than the park's. Parks inside cities, such as Sydney Harbour, stay the city.
const PARK_CLOSENESS: f64 = 3.0;

/// The national park or city that covers `point`, if any.
pub(super) fn park_or_city(index: &GeoIndex, point: Coordinates) -> Option<&Entry> {
    let city = covering_city(index, point);
    let park_radius = PlaceKind::NationalPark.match_radius_km()?;
    let is_park = |kind| kind == PlaceKind::NationalPark;
    let Some(park) = nearest(index, point, park_radius, is_park) else {
        return city.map(|(city, _)| city);
    };
    match city {
        Some((city, city_km))
            if city.population >= TOWN_MAX_POPULATION && park.1 * PARK_CLOSENESS >= city_km =>
        {
            Some(city)
        }
        _ => Some(park.0),
    }
}

/// The city whose reach covers `point` most deeply, with its distance. A suburb
/// wins only close to its own centre, so most of Sydney stays Sydney.
pub(super) fn covering_city(index: &GeoIndex, point: Coordinates) -> Option<(&Entry, f64)> {
    index
        .near(point, CITY_RADIUS_KM)
        .filter(|(entry, _)| entry.kind == PlaceKind::City)
        .filter_map(|(entry, distance)| {
            let depth = distance / city_reach_km(entry.population);
            (depth <= 1.0).then_some((entry, distance, depth))
        })
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(entry, distance, _)| (entry, distance))
}
