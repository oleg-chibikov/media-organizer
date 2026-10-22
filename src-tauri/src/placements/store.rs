use std::{
    path::{Path, PathBuf},
    sync::{Mutex, PoisonError},
};

use super::{
    storage::{load_placements, save_placements},
    PlacedSuggestion,
};

static PLACEMENTS_LOCK: Mutex<()> = Mutex::new(());

/// Placed suggestions persisted under `<appData>/placements`.
#[derive(Debug, Clone)]
pub(crate) struct PlacementStore {
    app_data_dir: PathBuf,
}

impl PlacementStore {
    pub(crate) fn in_app_data(app_data_dir: &Path) -> Self {
        Self {
            app_data_dir: app_data_dir.to_path_buf(),
        }
    }

    pub(crate) fn load(&self) -> Result<Vec<PlacedSuggestion>, String> {
        load_placements(&self.app_data_dir)
    }

    /// The records whose folder is still on disk, one check per record. The rest are
    /// dropped from the file, which keeps the registry from growing forever.
    pub(crate) fn load_existing(&self) -> Result<Vec<PlacedSuggestion>, String> {
        let _guard = PLACEMENTS_LOCK
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut records = self.load()?;
        let before = records.len();
        records.retain(|record| record.folder().is_dir());
        if records.len() != before {
            save_placements(&self.app_data_dir, records.clone())?;
        }
        Ok(records)
    }

    /// Applies `change` to the saved records; concurrent updates run one at a time.
    pub(crate) fn update(
        &self,
        change: impl FnOnce(&mut Vec<PlacedSuggestion>),
    ) -> Result<(), String> {
        let _guard = PLACEMENTS_LOCK
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut records = self.load()?;
        change(&mut records);
        save_placements(&self.app_data_dir, records)
    }
}
