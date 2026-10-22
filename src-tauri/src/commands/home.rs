use tauri::AppHandle;

use super::{app_dirs::settings_store, background::run_blocking};
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
