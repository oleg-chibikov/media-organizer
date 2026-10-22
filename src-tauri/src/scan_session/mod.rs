//! The records and dates of the scan on screen. Commands take file ids and look the files up
//! here, so the webview doesn't send every record and date back over IPC.

mod session_reader;

#[cfg(test)]
mod tests;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, PoisonError, RwLock},
};

use crate::models::{FileRecord, MetadataResult};
pub(crate) use session_reader::SessionReader;

const REPLACED: &str = "A newer scan replaced this one.";

#[derive(Default)]
pub(crate) struct ScanSession {
    records: RwLock<HashMap<String, FileRecord>>,
    metadata: RwLock<HashMap<String, MetadataResult>>,
}

impl ScanSession {
    pub(crate) fn add_records(&self, records: &[FileRecord]) {
        let mut stored = self.records.write().unwrap_or_else(PoisonError::into_inner);
        for record in records {
            stored.insert(record.id.clone(), record.clone());
        }
    }

    pub(crate) fn add_metadata(&self, results: &[MetadataResult]) {
        let mut stored = self
            .metadata
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        for result in results {
            stored.insert(result.file_id.clone(), result.clone());
        }
    }

    /// The records of `file_ids` in their order. Fails on an id the scan didn't find.
    pub(crate) fn records(&self, file_ids: &[String]) -> Result<Vec<FileRecord>, String> {
        let stored = self.records.read().unwrap_or_else(PoisonError::into_inner);
        file_ids
            .iter()
            .map(|file_id| {
                stored
                    .get(file_id)
                    .cloned()
                    .ok_or_else(|| format!("The scan has no file {file_id}."))
            })
            .collect()
    }

    /// The dates read so far for `file_ids`, in their order. Files still unread are left out.
    pub(crate) fn metadata(&self, file_ids: &[String]) -> Vec<MetadataResult> {
        let stored = self.metadata.read().unwrap_or_else(PoisonError::into_inner);
        file_ids
            .iter()
            .filter_map(|file_id| stored.get(file_id).cloned())
            .collect()
    }
}

/// The session of the latest scan. A new scan replaces it, and a command for an older
/// scan fails instead of mixing two folders.
#[derive(Default)]
pub(crate) struct CurrentScan {
    current: Mutex<Option<(u64, Arc<ScanSession>)>>,
}

impl CurrentScan {
    pub(crate) fn begin(&self, scan_id: u64) -> Arc<ScanSession> {
        let session = Arc::new(ScanSession::default());
        *self.lock() = Some((scan_id, session.clone()));
        session
    }

    pub(crate) fn session(&self, scan_id: u64) -> Result<Arc<ScanSession>, String> {
        match self.lock().as_ref() {
            Some((current_id, session)) if *current_id == scan_id => Ok(session.clone()),
            _ => Err(REPLACED.to_string()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<(u64, Arc<ScanSession>)>> {
        self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
