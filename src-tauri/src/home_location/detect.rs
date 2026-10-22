use crate::{
    file_places::{Home, PlaceScan},
    geo::Place,
    models::{FileRecord, HomeLocation, HomeSource, MetadataResult, PlaceDetection},
    settings::{self, SettingsStore},
};

/// The place folder of each file, with a country folder outside the saved home's country. A scan
/// that covers enough days learns home from the files and saves it over a time zone guess. Without a
/// saved home, a smaller scan infers home for itself.
pub(crate) fn detect_file_places(
    store: &SettingsStore,
    files: &[FileRecord],
    metadata_results: &[MetadataResult],
) -> Result<PlaceDetection, String> {
    let saved = store.load()?.home;
    let scan = PlaceScan::new(files, metadata_results);
    let home = match saved {
        Some(home) if home.source != HomeSource::TimeZone => Some(home_of(home)),
        guess => match scan.home_city() {
            Some(city) if city.worth_saving() => {
                learn(store, learned_home(&city.place));
                Some(city.home())
            }
            // A small scan is often one holiday, so a time zone guess beats its busiest city.
            inferred => guess.map(home_of).or(inferred.map(|city| city.home())),
        },
    };
    Ok(scan.detect(home.as_ref()))
}

fn home_of(home: HomeLocation) -> Home {
    Home {
        country: home.country,
    }
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
    match store.update(|current| settings::offer_home_location(current, home)) {
        Ok(_) => log::info!("Learned home: {name}"),
        Err(error) => log::warn!("Failed to save the learned home {name}: {error}"),
    }
}
