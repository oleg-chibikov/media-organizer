//! Tauri command boundary: thin handlers that validate input and delegate to backend modules.

pub(crate) mod destination;
pub(crate) mod history;
pub(crate) mod metadata;
pub(crate) mod move_execution;
pub(crate) mod plan;
pub(crate) mod scan;
pub(crate) mod system;
pub(crate) mod undo_redo;
