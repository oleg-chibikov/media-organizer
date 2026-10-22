//! Installs the latest GitHub release on start, with no prompt, then restarts the app.

use tauri::{AppHandle, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::app_state::AppState;

/// Checks for a newer release in the background. Dev builds skip the check.
pub fn update_on_start(app: &AppHandle) {
    if cfg!(debug_assertions) {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = update_to_latest(&app).await {
            log::warn!("Could not update the app: {error}");
        }
    });
}

async fn update_to_latest(app: &AppHandle) -> tauri_plugin_updater::Result<()> {
    let Some(update) = app.updater()?.check().await? else {
        log::info!("The app is up to date");
        return Ok(());
    };
    log::info!("Downloading version {}", update.version);
    show_progress(app, &update);
    let bytes = update.download(|_, _| {}, || {}).await?;

    // A restart would cut a move short, so wait for the next start once work begins.
    if !app.state::<AppState>().picked_folders.is_empty() {
        log::info!("Version {} installs on the next start", update.version);
        return Ok(());
    }
    // On Windows the installer closes the app here and opens the new version.
    update.install(bytes)?;
    log::info!("Installed version {}, restarting", update.version);
    app.restart();
}

fn show_progress(app: &AppHandle, update: &Update) {
    if let Some(window) = app.get_webview_window("main") {
        let title = format!(
            "{} {}, updating to {}",
            app.package_info().name,
            update.current_version,
            update.version
        );
        if let Err(error) = window.set_title(&title) {
            log::warn!("Could not set the window title: {error}");
        }
    }
}
