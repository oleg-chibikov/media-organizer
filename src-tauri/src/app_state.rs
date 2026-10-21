use crate::{metadata_queue::MetadataQueue, scanner::ActiveScan};

/// Process-wide state registered with Tauri and shared by command handlers.
#[derive(Default)]
pub struct AppState {
    pub(crate) active_scan: std::sync::Arc<ActiveScan>,
    pub(crate) metadata_queue: MetadataQueue,
}
