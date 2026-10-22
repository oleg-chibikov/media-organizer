//! The saved home: used for place suggestions, read from the device, learned
//! from large scans and picked by name in settings.

mod detect;
mod device;
mod device_home;
mod search;

#[cfg(test)]
mod tests;

pub(crate) use detect::detect_place_suggestions;
pub(crate) use device::SystemLocator;
pub(crate) use device_home::{find_home_from_device, use_current_location};
pub(crate) use search::search_home_cities;
