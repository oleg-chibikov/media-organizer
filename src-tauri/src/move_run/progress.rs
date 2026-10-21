use tauri::AppHandle;

use crate::{
    events,
    models::{MoveProgressEvent, MoveProgressPhase},
};

pub(crate) fn emit_move_progress_message(
    app: &AppHandle,
    phase: MoveProgressPhase,
    file_id: Option<&str>,
    message: &str,
    completed: u64,
    total: u64,
) {
    let _ = events::emit_move_progress(
        app,
        MoveProgressEvent {
            phase,
            file_id: file_id.map(str::to_string),
            message: message.to_string(),
            completed,
            total,
        },
    );
}
