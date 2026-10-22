// Test fixtures write files directly; production code is held to clippy.toml.
#![cfg_attr(test, allow(clippy::disallowed_methods))]

mod app_state;
mod commands;
mod console_title;
mod events;
mod file_ops;
mod filename;
// Used by the trips module next.
mod geo;
mod history;
mod home_location;
mod metadata;
mod metadata_queue;
mod models;
mod move_run;
mod mover;
mod path_safety;
mod picked_folders;
mod placements;
mod planner;
mod safe_fs;
mod scan_metadata;
mod scan_session;
mod scanner;
mod settings;
mod system_explorer;
mod trips;
mod undo_redo;

#[cfg(test)]
mod test_support;

use app_state::AppState;
use commands::{
    destination, history as history_commands, home, move_execution, places, plan, scan, system,
    undo_redo as undo_redo_commands,
};
use tauri::Manager;

const LOG_FILE_MAX_BYTES: u128 = 5 * 1024 * 1024;

/// Writes to the console and to a rotating file in the OS log folder.
fn log_plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri_plugin_log::Builder::new()
        .level(log::LevelFilter::Info)
        .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
        .max_file_size(LOG_FILE_MAX_BYTES)
        .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(5))
        .build()
}

fn log_startup(app: &tauri::App) {
    let version = &app.package_info().version;
    log::info!("Media Organizer {version} started");
    if let Ok(log_dir) = app.path().app_log_dir() {
        log::info!("Log files are in {}", log_dir.display());
    }
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("{info}");
        default_hook(info);
    }));
}

/// Shows the version in the window title and in the Windows console title.
fn show_version(app: &tauri::App) {
    let info = app.package_info();
    let title = format!("{} {}", info.name, info.version);
    if let Some(window) = app.get_webview_window("main") {
        if let Err(error) = window.set_title(&title) {
            log::warn!("Could not set the window title: {error}");
        }
    }
    console_title::set(&format!("{title} backend log"));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(log_plugin())
        .setup(|app| {
            log_startup(app);
            show_version(app);
            home::find_home_on_start(app.handle());
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            system::health_check,
            system::choose_folder,
            destination::get_destination_settings,
            destination::get_effective_destination_root,
            destination::choose_root_media_folder,
            destination::clear_root_media_folder,
            destination::set_force_processed_folder,
            destination::set_rename_in_place,
            destination::set_group_by_place,
            scan::start_scan,
            scan::cancel_scan,
            scan::set_metadata_paused,
            plan::apply_place_suggestions,
            plan::clear_scan_plans,
            plan::get_move_confirmation_summary,
            places::detect_place_suggestions,
            home::search_home_cities,
            home::set_home_location,
            home::use_current_location,
            history_commands::delete_history_run,
            history_commands::list_history_runs,
            history_commands::get_history_run,
            undo_redo_commands::undo_history_run,
            undo_redo_commands::redo_history_run,
            move_execution::execute_move_plan,
            move_execution::pause_move_run,
            move_execution::remove_empty_source_folders,
            system::reveal_file_in_explorer,
            system::open_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
