//! Remembers each run's refreshed index entry until its log changes, so listing history
//! reads only the logs that grew since the last list instead of every log every time.

use std::{
    collections::BTreeMap,
    fs,
    sync::{Mutex, MutexGuard, PoisonError},
    time::SystemTime,
};

use crate::models::HistoryIndexEntry;

/// Logs only grow by appends, so a new size or write time means new records.
#[derive(PartialEq)]
struct LogStamp {
    len: u64,
    modified: Option<SystemTime>,
}

static SUMMARIES: Mutex<BTreeMap<String, (LogStamp, HistoryIndexEntry)>> =
    Mutex::new(BTreeMap::new());

/// Returns the cached entry while the run's log is unchanged, otherwise calls `refresh`.
pub(super) fn cached_entry(
    entry: &HistoryIndexEntry,
    refresh: impl FnOnce(&HistoryIndexEntry) -> HistoryIndexEntry,
) -> HistoryIndexEntry {
    let Some(stamp) = log_stamp(&entry.history_file_path) else {
        return refresh(entry);
    };
    let key = cache_key(entry);
    if let Some((cached_stamp, cached)) = summaries().get(&key) {
        if *cached_stamp == stamp {
            return cached.clone();
        }
    }
    let refreshed = refresh(entry);
    summaries().insert(key, (stamp, refreshed.clone()));
    refreshed
}

/// Drops cached runs that left the index.
pub(super) fn keep_only(entries: &[HistoryIndexEntry]) {
    let keys: Vec<String> = entries.iter().map(cache_key).collect();
    summaries().retain(|key, _| keys.contains(key));
}

fn cache_key(entry: &HistoryIndexEntry) -> String {
    format!("{}|{}", entry.run_id, entry.history_file_path)
}

fn log_stamp(path: &str) -> Option<LogStamp> {
    let metadata = fs::metadata(path).ok()?;
    Some(LogStamp {
        len: metadata.len(),
        modified: metadata.modified().ok(),
    })
}

fn summaries() -> MutexGuard<'static, BTreeMap<String, (LogStamp, HistoryIndexEntry)>> {
    SUMMARIES.lock().unwrap_or_else(PoisonError::into_inner)
}
