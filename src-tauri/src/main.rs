mod app_state;
mod atomic_file;
mod commands;
mod events;
mod file_ops;
mod filename;
mod history;
mod metadata;
mod metadata_queue;
mod models;
mod move_run;
mod mover;
mod path_safety;
mod planner;
mod safe_fs;
mod scanner;
mod settings;
mod system_explorer;
mod undo_redo;

#[cfg(test)]
mod test_support;

use app_state::AppState;
use commands::{
    destination, history as history_commands, metadata as metadata_commands, move_execution, plan,
    scan, system, undo_redo as undo_redo_commands,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            system::health_check,
            system::get_processing_config,
            system::choose_folder,
            destination::get_destination_settings,
            destination::choose_root_media_folder,
            destination::clear_root_media_folder,
            destination::set_force_processed_folder,
            scan::start_scan,
            scan::cancel_scan,
            metadata_commands::read_metadata_for_file,
            metadata_commands::read_metadata_for_files,
            plan::create_move_plan,
            plan::get_move_confirmation_summary,
            history_commands::delete_history_run,
            history_commands::list_history_runs,
            history_commands::get_history_run,
            undo_redo_commands::undo_history_run,
            undo_redo_commands::redo_history_run,
            move_execution::execute_move_plan,
            system::reveal_file_in_explorer
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
