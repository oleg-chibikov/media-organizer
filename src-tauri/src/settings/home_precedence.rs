use super::updates::stamped;
use crate::models::{DestinationSettings, HomeLocation, HomeSource};

/// Saves a home found on the device or in the files when its source outranks
/// the saved one, so it overwrites no stronger choice.
pub(crate) fn offer_home_location(
    settings: &DestinationSettings,
    home: HomeLocation,
) -> DestinationSettings {
    let outranks = settings
        .home
        .as_ref()
        .is_none_or(|saved| rank(home.source) > rank(saved.source));
    if !outranks {
        return settings.clone();
    }
    replace_home_location(settings, home)
}

/// Saves `home` as it is, for a lookup the person asked for.
pub(crate) fn replace_home_location(
    settings: &DestinationSettings,
    home: HomeLocation,
) -> DestinationSettings {
    stamped(DestinationSettings {
        home: Some(home),
        ..settings.clone()
    })
}

fn rank(source: HomeSource) -> u8 {
    match source {
        HomeSource::TimeZone => 0,
        HomeSource::Learned => 1,
        HomeSource::Device => 2,
        HomeSource::Manual => 3,
    }
}
