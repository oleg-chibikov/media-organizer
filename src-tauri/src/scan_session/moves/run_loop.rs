use std::time::Duration;

use tokio_util::sync::CancellationToken;

use super::Moves;
use crate::{
    events::{self, EventSink},
    history::HistoryStore,
    models::MovesDoneEvent,
    move_run,
    scan_session::ScanSession,
};

// New plans wake the run at once. This only bounds a wake-up that got lost.
const RECHECK: Duration = Duration::from_millis(250);

enum Next {
    Batch(Vec<String>, CancellationToken),
    Done,
    Stopped,
}

impl ScanSession {
    pub(super) fn run_moves(&self, sink: &impl EventSink, history: &HistoryStore) {
        loop {
            match self.next_batch() {
                Next::Batch(file_ids, pause) => {
                    let reached = self.move_batch(sink, history, &file_ids, &pause);
                    let mut moves = self.moves();
                    moves.batch = None;
                    match reached {
                        Ok(reached) => moves.done.extend(reached),
                        Err(error) => return self.report_done(sink, false, Some(error)),
                    }
                }
                Next::Done => return self.clean_source(sink),
                Next::Stopped => return,
            }
        }
    }

    /// Waits until ready files wait, the scan has nothing left to check, or the run stops.
    fn next_batch(&self) -> Next {
        let mut moves = self.moves();
        loop {
            if moves.stopped {
                return Next::Stopped;
            }
            if !moves.paused {
                let ready = self.ready_file_ids(&moves.done);
                if !ready.is_empty() {
                    let pause = CancellationToken::new();
                    moves.batch = Some(pause.clone());
                    return Next::Batch(ready, pause);
                }
                if !self.still_checking() {
                    return Next::Done;
                }
            }
            moves = self.wait_for_change(moves);
        }
    }

    fn wait_for_change<'a>(
        &self,
        moves: std::sync::MutexGuard<'a, Moves>,
    ) -> std::sync::MutexGuard<'a, Moves> {
        self.moves_changed
            .wait_timeout(moves, RECHECK)
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .0
    }

    fn clean_source(&self, sink: &impl EventSink) {
        let context = &self.context;
        match move_run::remove_empty_source_folders(&context.settings, &context.folder) {
            Ok(removed) => self.report_done(sink, removed, None),
            Err(error) => self.report_done(
                sink,
                false,
                Some(format!("Failed to remove empty source folders: {error}")),
            ),
        }
    }

    fn report_done(&self, sink: &impl EventSink, removed: bool, error: Option<String>) {
        let scan_id = self.scan_id;
        events::moves_done(
            sink,
            MovesDoneEvent {
                scan_id,
                removed_selected_folder: removed,
                error,
            },
        );
    }
}
