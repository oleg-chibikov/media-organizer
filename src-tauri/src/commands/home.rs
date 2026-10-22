use tauri::AppHandle;

use super::{
    app_dirs::settings_store,
    background::{run_blocking, spawn_job},
};
use crate::{
    home_location,
    models::{DestinationSettings, HomeCandidate, HomeLocation},
    settings,
};

/// Cities to pick home from. The first search loads the place table.
#[tauri::command]
pub async fn search_home_cities(query: String) -> Result<Vec<HomeCandidate>, String> {
    run_blocking(move || Ok(home_location::search_home_cities(&query))).await
}

/// Saves the picked home, or clears it so the next large scan learns it.
#[tauri::command]
pub async fn set_home_location(
    home: Option<HomeLocation>,
    app: AppHandle,
) -> Result<DestinationSettings, String> {
    let store = settings_store(&app)?;
    run_blocking(move || store.update(|current| settings::set_home_location(current, home))).await
}

/// Saves the city at the device's location as home. Waits up to 10 s for a fix.
#[tauri::command]
pub async fn use_current_location(app: AppHandle) -> Result<DestinationSettings, String> {
    let store = settings_store(&app)?;
    run_blocking(move || home_location::use_current_location(&store, &home_location::SystemLocator))
        .await
}

/// Looks for home on the device in the background when none is saved.
pub fn find_home_on_start(app: &AppHandle) {
    match settings_store(app) {
        Ok(store) => spawn_job(move || {
            home_location::find_home_from_device(&store, &home_location::SystemLocator)
        }),
        Err(error) => log::warn!("Skipped the device home: {error}"),
    }
}
