//! Names a point by the broad area it sits in: a park or a city.

use super::index::{Entry, GeoIndex};
use super::lookup::nearest;
use super::nearby::Nearby;
use super::parks::{Park, Parks};
use super::place::Place;
use super::radius::{CITY_RADIUS_KM, FALLBACK_RADIUS_KM};

/// Cities this big keep their name over small parks inside them.
const BIG_CITY_MIN_POPULATION: u32 = 50_000;
/// Inside a big city, a park this large still wins, as Royal National Park does in Sydney.
const LARGE_PARK_MIN_KM2: f32 = 100.0;
/// Away from big cities, a town this close to a large park takes the park's
/// name, as Katoomba takes Blue Mountains National Park.
const TOWN_PULL_KM: f64 = 5.0;

pub(super) enum Area<'a> {
    Park(&'a Park),
    City(&'a Entry),
}

/// The park or city that covers the point, if any.
pub(super) fn park_or_city<'a>(nearby: &Nearby<'a>, parks: &'a Parks) -> Option<Area<'a>> {
    let point = nearby.point;
    let in_big_city = nearby
        .covering_cities()
        .any(|(city, _)| city.population >= BIG_CITY_MIN_POPULATION);
    let park = parks.containing(point);
    if let Some(park) = park.filter(|park| !in_big_city || is_large(park)) {
        return Some(Area::Park(park));
    }
    let nearby_large_park = || {
        parks
            .near(point, TOWN_PULL_KM)
            .filter(|(park, _)| is_large(park))
            .min_by(|a, b| a.1.total_cmp(&b.1))
    };
    if let Some((park, _)) = nearby_large_park().filter(|_| !in_big_city) {
        return Some(Area::Park(park));
    }
    covering_city(nearby).map(Area::City)
}

/// The city whose reach covers the point most deeply. A suburb wins only close
/// to its own centre, so most of Sydney stays Sydney.
pub(super) fn covering_city<'a>(nearby: &Nearby<'a>) -> Option<&'a Entry> {
    nearby
        .covering_cities()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(city, _)| city)
}

/// A park as a place. Its country and region come from the GeoNames place
/// nearest its label point.
pub(super) fn park_place(index: &GeoIndex, park: &Park) -> Option<Place> {
    // Most parks have a place within the small radius, which is quicker to search.
    let (near, _) = [CITY_RADIUS_KM, FALLBACK_RADIUS_KM]
        .into_iter()
        .find_map(|radius| nearest(index, park.label, radius, |_| true))?;
    Some(Place {
        name: park.name.clone(),
        kind: park.kind,
        latitude: park.label.latitude,
        longitude: park.label.longitude,
        population: 0,
        ..index.place(near)
    })
}

impl Area<'_> {
    pub(super) fn place(&self, index: &GeoIndex) -> Option<Place> {
        match self {
            Area::Park(park) => park_place(index, park),
            Area::City(city) => Some(index.place(city)),
        }
    }
}

fn is_large(park: &Park) -> bool {
    park.area_km2 >= LARGE_PARK_MIN_KM2
}
