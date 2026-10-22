//! Home from the device: one location fix, else the system time zone.

#[cfg(target_os = "macos")]
mod core_location;
mod time_zone;
mod zone_countries;

#[cfg(test)]
pub(super) use zone_countries::country_in_table;

use crate::{
    geo::{self, Coordinates, Place},
    models::{HomeLocation, HomeSource},
};

/// Where the device is. Tests swap in a fake so they don't ask CoreLocation.
pub(crate) trait DeviceLocator {
    /// One location fix, or `None` when denied, unavailable or too slow.
    fn current_point(&self) -> Option<Coordinates>;
    /// The IANA zone name, such as `Australia/Sydney`.
    fn time_zone(&self) -> Option<String>;
}

pub(crate) struct SystemLocator;

impl DeviceLocator for SystemLocator {
    #[cfg(target_os = "macos")]
    fn current_point(&self) -> Option<Coordinates> {
        core_location::current_point(std::time::Duration::from_secs(10))
    }

    #[cfg(not(target_os = "macos"))]
    fn current_point(&self) -> Option<Coordinates> {
        None
    }

    fn time_zone(&self) -> Option<String> {
        iana_time_zone::get_timezone().ok()
    }
}

/// The city at the device's location, else the city its time zone names.
pub(super) fn device_home(locator: &impl DeviceLocator) -> Option<HomeLocation> {
    located_home(locator).or_else(|| time_zone_home(locator))
}

/// The city nearest the device's location fix.
pub(super) fn located_home(locator: &impl DeviceLocator) -> Option<HomeLocation> {
    let point = locator.current_point()?;
    let city = geo::nearest_city(point).or_else(|| geo::nearest_place(point))?;
    Some(home_at(&city, HomeSource::Device))
}

fn time_zone_home(locator: &impl DeviceLocator) -> Option<HomeLocation> {
    let zone = locator.time_zone()?;
    let country = zone_countries::country_of_zone(&zone);
    let city = time_zone::city_for_zone(&zone, country.as_deref())?;
    Some(home_at(&city, HomeSource::TimeZone))
}

fn home_at(city: &Place, source: HomeSource) -> HomeLocation {
    HomeLocation {
        name: city.name.clone(),
        country: city.country_name.clone(),
        latitude: city.latitude,
        longitude: city.longitude,
        source,
    }
}
