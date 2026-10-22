use tokio_util::sync::CancellationToken;

use crate::{
    events::EventSink,
    history::HistoryStore,
    move_run::{self, MoveBatch},
    scan_session::ScanSession,
};

impl ScanSession {
    /// Moves one batch through `move_run`, which checks, moves and logs each file.
    /// Returns the files that got a result.
    pub(super) fn move_batch(
        &self,
        sink: &impl EventSink,
        history: &HistoryStore,
        file_ids: &[String],
        pause: &CancellationToken,
    ) -> Result<Vec<String>, String> {
        let batch = MoveBatch {
            files: self.records(file_ids)?,
            metadata_results: self.metadata(file_ids),
            plans: self.plans(file_ids)?,
        };
        let context = &self.context;
        let run = move_run::start_move_run(
            sink,
            &context.settings,
            history,
            &context.placements,
            &context.folder,
            batch,
        )?;
        Ok(run.execute(sink, pause))
    }
}
