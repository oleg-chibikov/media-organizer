use crate::{
    geo::{Coordinates, Place},
    models::{FileRecord, HomeLocation, HomeSource, MetadataResult, PlaceDetection},
    settings::{self, SettingsStore},
    trips::{Home, PlaceScan},
};

/// Trips and outings measured from the saved home, with file place labels and a status. With no saved home, home is
/// inferred from the files, and saved when the scan covers enough days.
pub(crate) fn detect_place_suggestions(
    store: &SettingsStore,
    files: &[FileRecord],
    metadata_results: &[MetadataResult],
) -> Result<PlaceDetection, String> {
    let saved = store.load()?.home;
    let scan = PlaceScan::new(files, metadata_results);
    let home = match saved {
        Some(home) => Some(Home {
            name: home.name,
            point: Coordinates::new(home.latitude, home.longitude),
            country: home.country,
        }),
        None => scan.home_city().map(|city| {
            if city.worth_saving() {
                learn(store, learned_home(&city.place));
            }
            city.home()
        }),
    };
    Ok(scan.detect(home.as_ref()))
}

fn learned_home(city: &Place) -> HomeLocation {
    HomeLocation {
        name: city.name.clone(),
        country: city.country_name.clone(),
        latitude: city.latitude,
        longitude: city.longitude,
        source: HomeSource::Learned,
    }
}

// A failed save only means home is inferred again next time.
fn learn(store: &SettingsStore, home: HomeLocation) {
    let name = home.name.clone();
    match store.update(|current| settings::learn_home_location(current, home)) {
        Ok(_) => log::info!("Learned home: {name}"),
        Err(error) => log::warn!("Failed to save the learned home {name}: {error}"),
    }
}
