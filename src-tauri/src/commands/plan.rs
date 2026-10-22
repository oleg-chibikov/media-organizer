use std::path::PathBuf;

use tauri::AppHandle;

use super::{
    app_dirs::{placement_store, settings_store},
    background::{run_blocking, spawn_job},
    session::scan_session,
};
use crate::{
    models::{MoveConfirmationSummary, MovePlan, PlaceSuggestion, PlacedMove},
    mover, path_safety, planner,
};

#[tauri::command]
pub async fn create_move_plan(
    request_id: u64,
    selected_path: String,
    scan_id: u64,
    file_ids: Vec<String>,
    place_suggestions: Vec<PlaceSuggestion>,
    app: AppHandle,
) -> Result<(), String> {
    let session = scan_session(&app, scan_id)?;
    let files = session.records(&file_ids)?;
    let metadata_results = session.metadata(&file_ids);
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
        planner::stream_move_plan(
            &app,
            request_id,
            &layout,
            &files,
            &metadata_results,
            &place_folders,
        );
    });
    Ok(())
}

/// Files earlier runs placed in the folders the accepted suggestions continue, planned
/// into their new day folders. Files whose day folder stays the same are left out.
#[tauri::command]
pub async fn plan_continued_moves(
    selected_path: String,
    scan_id: u64,
    file_ids: Vec<String>,
    place_suggestions: Vec<PlaceSuggestion>,
    app: AppHandle,
) -> Result<Vec<PlacedMove>, String> {
    let session = scan_session(&app, scan_id)?;
    let metadata_results = session.metadata(&file_ids);
    let settings = settings_store(&app)?;
    let placements = placement_store(&app)?;
    let moves = run_blocking(move || {
        planner::plan_continued_moves(
            &settings,
            &placements,
            &PathBuf::from(selected_path),
            &place_suggestions,
            &metadata_results,
        )
    })
    .await?;
    // Placed files join the scan, so later plans and moves can find them by id.
    let records: Vec<_> = moves.iter().map(|placed| placed.record.clone()).collect();
    let metadata: Vec<_> = moves.iter().map(|placed| placed.metadata.clone()).collect();
    session.add_records(&records);
    session.add_metadata(&metadata);
    Ok(moves)
}

#[tauri::command]
pub fn get_move_confirmation_summary(
    scan_id: u64,
    plans: Vec<MovePlan>,
    app: AppHandle,
) -> Result<MoveConfirmationSummary, String> {
    let session = scan_session(&app, scan_id)?;
    let file_ids: Vec<String> = plans.iter().map(|plan| plan.file_id.clone()).collect();
    let files = session.records(&file_ids)?;
    let metadata_results = session.metadata(&file_ids);
    Ok(mover::build_move_confirmation_summary(
        &files,
        &metadata_results,
        &plans,
    ))
}
