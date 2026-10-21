use crate::{
    events::{self, EventSink},
    models::{MoveProgressEvent, MoveProgressPhase},
};

pub(super) fn emit_move_progress_message(
    sink: &impl EventSink,
    phase: MoveProgressPhase,
    file_id: Option<&str>,
    message: &str,
    completed: u64,
    total: u64,
) {
    events::move_progress(
        sink,
        MoveProgressEvent {
            phase,
            file_id: file_id.map(str::to_string),
            message: message.to_string(),
            completed,
            total,
        },
    );
}
