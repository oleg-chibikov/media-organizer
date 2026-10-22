//! Names home after the city in the system time zone, such as Sydney for
//! `Australia/Sydney`.

use crate::geo::{self, Place};

const CANDIDATES: usize = 50;

/// The biggest city named like the zone's last part, in `country_code` when
/// it is known. Zones such as `Etc/UTC` name no city.
pub(super) fn city_for_zone(zone: &str, country_code: Option<&str>) -> Option<Place> {
    let (region, city) = zone.rsplit_once('/')?;
    if region.starts_with("Etc") {
        return None;
    }
    let name = city.replace('_', " ");
    let candidates = geo::search_cities(&name, CANDIDATES);
    match country_code {
        // Starts-with matches come first, so `Ho Chi Minh` finds `Ho Chi Minh City`.
        Some(code) => candidates
            .into_iter()
            .find(|place| place.country_code.eq_ignore_ascii_case(code)),
        None => candidates
            .into_iter()
            .find(|place| place.name.eq_ignore_ascii_case(&name)),
    }
}
