mod commands;
mod events;
mod filename;
mod logging;
mod metadata;
mod models;
mod mover;
mod planner;
mod scanner;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::health_check,
            commands::choose_folder,
            commands::start_scan,
            commands::cancel_scan,
            commands::read_metadata_for_file,
            commands::create_move_plan
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
