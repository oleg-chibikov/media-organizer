use std::path::PathBuf;

use tauri::AppHandle;

use super::{
    app_dirs::{placement_store, settings_store},
    background::{run_blocking, spawn_job},
};
use crate::{
    models::{
        FileRecord, MetadataResult, MoveConfirmationSummary, MovePlan, PlaceSuggestion, PlacedMove,
    },
    mover, path_safety, planner,
};

#[tauri::command]
pub async fn create_move_plan(
    selected_path: String,
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
    place_suggestions: Vec<PlaceSuggestion>,
    app: AppHandle,
) -> Result<(), String> {
    let settings = settings_store(&app)?;
    let placements = placement_store(&app)?;
    let with_places = !place_suggestions.is_empty();
    let (layout, placed, files) = run_blocking(move || {
        let selected_folder = PathBuf::from(selected_path);
        path_safety::validate_selected_folder(&selected_folder)?;
        let layout =
            planner::DestinationLayout::for_folder(&settings, &selected_folder, with_places)?;
        let placed = planner::placed_files_under(&settings, &placements, &selected_folder, &files);
        Ok::<_, String>((layout, placed, files))
    })
    .await?;
    let place_folders =
        planner::PlaceFolders::new(&place_suggestions, &metadata_results)?.with_placed(placed);

    spawn_job(move || {
        planner::stream_move_plan(&app, &layout, &files, &metadata_results, &place_folders);
    });
    Ok(())
}

/// Files earlier runs placed in the folders the accepted suggestions continue, planned
/// into their new day folders. Files whose day folder stays the same are left out.
#[tauri::command]
pub async fn plan_continued_moves(
    selected_path: String,
    place_suggestions: Vec<PlaceSuggestion>,
    metadata_results: Vec<MetadataResult>,
    app: AppHandle,
) -> Result<Vec<PlacedMove>, String> {
    let settings = settings_store(&app)?;
    let placements = placement_store(&app)?;
    run_blocking(move || {
        planner::plan_continued_moves(
            &settings,
            &placements,
            &PathBuf::from(selected_path),
            &place_suggestions,
            &metadata_results,
        )
    })
    .await
}

#[tauri::command]
pub fn get_move_confirmation_summary(
    files: Vec<FileRecord>,
    metadata_results: Vec<MetadataResult>,
    plans: Vec<MovePlan>,
) -> MoveConfirmationSummary {
    mover::build_move_confirmation_summary(&files, &metadata_results, &plans)
}
