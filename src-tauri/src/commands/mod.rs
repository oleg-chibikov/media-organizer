//! Tauri command boundary: thin handlers that resolve app paths and delegate to backend modules.
//!
//! Filesystem work runs through `background`, so no handler blocks the UI thread.

mod app_dirs;
mod background;
mod folder_dialog;

pub(crate) mod destination;
pub(crate) mod history;
pub(crate) mod metadata;
pub(crate) mod move_execution;
pub(crate) mod places;
pub(crate) mod plan;
pub(crate) mod scan;
pub(crate) mod system;
pub(crate) mod undo_redo;
