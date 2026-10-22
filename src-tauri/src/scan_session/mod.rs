//! The records, dates and plans of the scan on screen. Commands take file ids and look the
//! files up here, so the webview doesn't send every record and date back over IPC.

mod moves;
mod planning;
mod session_reader;

#[cfg(test)]
mod tests;

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex, PoisonError, RwLock,
    },
};

use crate::models::{FileRecord, MetadataResult};
use moves::Moves;
pub(crate) use planning::PlanContext;
use planning::Planning;
pub(crate) use session_reader::SessionReader;

const REPLACED: &str = "A newer scan replaced this one.";

pub(crate) struct ScanSession {
    scan_id: u64,
    context: PlanContext,
    records: RwLock<HashMap<String, FileRecord>>,
    metadata: RwLock<HashMap<String, MetadataResult>>,
    planning: Mutex<Planning>,
    moves: Mutex<Moves>,
    /// Wakes a waiting move run when plans, dates or the scan change.
    moves_changed: Condvar,
    scanned: AtomicBool,
    dates_read: AtomicBool,
}

impl ScanSession {
    fn new(scan_id: u64, context: PlanContext) -> Self {
        Self {
            scan_id,
            context,
            records: RwLock::default(),
            metadata: RwLock::default(),
            planning: Mutex::default(),
            moves: Mutex::default(),
            moves_changed: Condvar::new(),
            scanned: AtomicBool::new(false),
            dates_read: AtomicBool::new(false),
        }
    }

    pub(crate) fn folder(&self) -> &std::path::Path {
        &self.context.folder
    }

    /// The scan walked the whole folder, or stopped.
    pub(crate) fn mark_scanned(&self) {
        self.scanned.store(true, Ordering::SeqCst);
        self.moves_changed.notify_all();
    }

    /// Every photo and video of the scan has a date, or the reads stopped.
    pub(crate) fn mark_dates_read(&self) {
        self.dates_read.store(true, Ordering::SeqCst);
        self.moves_changed.notify_all();
    }

    /// Files can still become ready: the scan, its reads or its place folders aren't done.
    fn still_checking(&self) -> bool {
        !self.scanned.load(Ordering::SeqCst)
            || !self.dates_read.load(Ordering::SeqCst)
            || self.waiting_for_places()
    }

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

    /// The records of the `file_ids` the scan knows, in their order.
    fn known_records(&self, file_ids: &[String]) -> Vec<FileRecord> {
        let stored = self.records.read().unwrap_or_else(PoisonError::into_inner);
        file_ids
            .iter()
            .filter_map(|id| stored.get(id).cloned())
            .collect()
    }

    fn metadata_of(&self, file_id: &str) -> Option<MetadataResult> {
        let stored = self.metadata.read().unwrap_or_else(PoisonError::into_inner);
        stored.get(file_id).cloned()
    }

    fn all_metadata(&self) -> Vec<MetadataResult> {
        let stored = self.metadata.read().unwrap_or_else(PoisonError::into_inner);
        stored.values().cloned().collect()
    }

    /// Every file with a date read.
    fn read_file_ids(&self) -> Vec<String> {
        let stored = self.metadata.read().unwrap_or_else(PoisonError::into_inner);
        stored.keys().cloned().collect()
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
    pub(crate) fn begin(&self, scan_id: u64, context: PlanContext) -> Arc<ScanSession> {
        let session = Arc::new(ScanSession::new(scan_id, context));
        if let Some((_, replaced)) = self.lock().replace((scan_id, session.clone())) {
            replaced.stop_moves();
        }
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
