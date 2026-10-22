//! The confirmed move run of a scan. It moves every ready file, then the files that become
//! ready while the scan is still checked, and cleans the source folders once all are done.

mod batch;
mod run_loop;

use std::{
    collections::HashSet,
    sync::{Arc, MutexGuard, PoisonError},
};

use tokio_util::sync::CancellationToken;

use super::ScanSession;
use crate::{events::EventSink, history::HistoryStore};

#[derive(Default)]
pub(super) struct Moves {
    started: bool,
    paused: bool,
    /// A newer scan replaced this one, so its run stops after the file moving now.
    stopped: bool,
    /// Files that got a result. A paused batch leaves the ones it didn't reach out.
    done: HashSet<String>,
    batch: Option<CancellationToken>,
}

impl ScanSession {
    /// Starts moving the scan's ready files on a thread of its own. Fails when a run of
    /// this scan already started.
    pub(crate) fn start_moves<S>(
        self: &Arc<Self>,
        sink: S,
        history: HistoryStore,
    ) -> Result<(), String>
    where
        S: EventSink + Send + Sync + 'static,
    {
        let mut moves = self.moves();
        if moves.started {
            return Err("The files of this scan are already moving.".to_string());
        }
        moves.started = true;
        let session = self.clone();
        std::thread::spawn(move || session.run_moves(&sink, &history));
        Ok(())
    }

    /// Stops the run after the file moving now. Files it didn't reach wait for the resume.
    pub(crate) fn pause_moves(&self) {
        let mut moves = self.moves();
        moves.paused = true;
        if let Some(batch) = &moves.batch {
            batch.cancel();
        }
    }

    pub(crate) fn resume_moves(&self) {
        self.moves().paused = false;
        self.moves_changed.notify_all();
    }

    pub(super) fn stop_moves(&self) {
        let mut moves = self.moves();
        moves.stopped = true;
        if let Some(batch) = &moves.batch {
            batch.cancel();
        }
        drop(moves);
        self.moves_changed.notify_all();
    }

    fn moves(&self) -> MutexGuard<'_, Moves> {
        self.moves.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
