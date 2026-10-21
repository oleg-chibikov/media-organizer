use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;

use crate::metadata_queue::MetadataQueue;

/// Process-wide state registered with Tauri and shared by command handlers.
#[derive(Default)]
pub struct AppState {
    pub(crate) scan_cancellation_token: Arc<Mutex<Option<CancellationToken>>>,
    pub(crate) metadata_queue: MetadataQueue,
}
