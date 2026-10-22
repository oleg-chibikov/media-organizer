use std::sync::Arc;

use crate::{
    metadata_queue::MetadataQueue, move_run::MovePause, picked_folders::PickedFolders,
    scan_metadata::MetadataPause, scan_session::CurrentScan, scanner::ActiveScan,
};

/// Process-wide state registered with Tauri and shared by command handlers.
#[derive(Default)]
pub struct AppState {
    pub(crate) active_scan: Arc<ActiveScan>,
    pub(crate) metadata_queue: Arc<MetadataQueue>,
    pub(crate) metadata_pause: Arc<MetadataPause>,
    pub(crate) current_scan: CurrentScan,
    pub(crate) move_pause: MovePause,
    pub(crate) picked_folders: PickedFolders,
}
