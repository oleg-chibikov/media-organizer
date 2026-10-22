//! Search distances. GeoNames gives one point per place and no outline,
//! so each kind gets a radius that roughly covers its usual size.

use super::kind::PlaceKind;

pub(super) const CITY_RADIUS_KM: f64 = 50.0;
pub(super) const FALLBACK_RADIUS_KM: f64 = 150.0;

const SMALL_SITE_RADIUS_KM: f64 = 1.0;
const CITY_PARK_RADIUS_KM: f64 = 1.5;
// Being near a summit means standing on it. Valleys below are often closer than 3 km.
const SUMMIT_RADIUS_KM: f64 = 1.0;
const NATURE_SPOT_RADIUS_KM: f64 = 3.0;
const LANDFORM_RADIUS_KM: f64 = 5.0;
const ISLAND_RADIUS_KM: f64 = 10.0;
const RESERVE_RADIUS_KM: f64 = 15.0;
const NATIONAL_PARK_RADIUS_KM: f64 = 40.0;

/// The widest radius below. Searches for points of interest start here.
pub(super) const MAX_POI_RADIUS_KM: f64 = NATIONAL_PARK_RADIUS_KM;

// Gives San Francisco about 18 km and a town of 5,000 about 1.4 km.
const CITY_REACH_KM_PER_SQRT_PERSON: f64 = 0.02;

/// How far a city spreads from its point, growing with population. Capped at
/// `CITY_RADIUS_KM`, so a search that wide finds every covering city.
pub(super) fn city_reach_km(population: u32) -> f64 {
    (CITY_REACH_KM_PER_SQRT_PERSON * f64::from(population).sqrt()).min(CITY_RADIUS_KM)
}

impl PlaceKind {
    /// How far from its point a visit still counts as inside the place.
    /// Cities return `None`: they are the fallback, matched by distance alone.
    pub(super) fn match_radius_km(self) -> Option<f64> {
        use PlaceKind::*;
        Some(match self {
            City => return None,
            Garden | Zoo | Museum | Castle | Palace | Monument | HistoricSite | Ruin | Temple => {
                SMALL_SITE_RADIUS_KM
            }
            Park | AmusementPark => CITY_PARK_RADIUS_KM,
            Peak => SUMMIT_RADIUS_KM,
            Beach | Waterfall | Lake => NATURE_SPOT_RADIUS_KM,
            Volcano | Canyon | Glacier => LANDFORM_RADIUS_KM,
            Island => ISLAND_RADIUS_KM,
            Reserve => RESERVE_RADIUS_KM,
            NationalPark => NATIONAL_PARK_RADIUS_KM,
        })
    }

    /// Lower levels sit inside higher ones and win when both match. The level
    /// follows the radius, except an island, which can hold any other place.
    pub(super) fn nesting_level(self) -> u8 {
        use PlaceKind::*;
        match self {
            Garden | Zoo | Museum | Castle | Palace | Monument | HistoricSite | Ruin | Temple
            | Peak => 0,
            Park | AmusementPark => 1,
            Beach | Waterfall | Lake => 2,
            Volcano | Canyon | Glacier => 3,
            Reserve => 4,
            NationalPark => 5,
            Island | City => 6,
        }
    }
}
