use super::device::{device_home, located_home, DeviceLocator};
use crate::{
    models::{DestinationSettings, HomeSource},
    settings::{self, SettingsStore},
};

const NO_FIX: &str = concat!(
    "Couldn't get the current location. ",
    "Check that Location Services allow Media Organizer in System Settings."
);

/// Saves a home from the device when none is saved, or only a time zone
/// guess is. Detection doesn't wait for it and uses whatever home is saved.
pub(crate) fn find_home_from_device(store: &SettingsStore, locator: &impl DeviceLocator) {
    let needed = match store.load() {
        Ok(settings) => settings
            .home
            .is_none_or(|home| home.source == HomeSource::TimeZone),
        Err(error) => {
            log::warn!("Skipped the device home: {error}");
            return;
        }
    };
    if !needed {
        return;
    }
    let Some(home) = device_home(locator) else {
        log::info!("Found no home from the device location or time zone");
        return;
    };
    let name = home.name.clone();
    match store.update(|current| settings::offer_home_location(current, home)) {
        Ok(_) => log::info!("Home from the device: {name}"),
        Err(error) => log::warn!("Failed to save the device home {name}: {error}"),
    }
}

/// Saves the city at the device's location as home, over any saved home,
/// because the person asked for it.
pub(crate) fn use_current_location(
    store: &SettingsStore,
    locator: &impl DeviceLocator,
) -> Result<DestinationSettings, String> {
    let home = located_home(locator).ok_or(NO_FIX)?;
    store.update(|current| settings::replace_home_location(current, home))
}
