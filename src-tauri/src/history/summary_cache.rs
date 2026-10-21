//! Remembers each run's refreshed index entry until its log changes, so listing history
//! reads only the logs that grew since the last list instead of every log every time.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, PoisonError},
    time::SystemTime,
};

use super::run_lease;
use crate::models::HistoryIndexEntry;

/// Logs only grow by appends, so a new size or write time means new records. A run that
/// stops being worked on changes status without a write, so that counts too.
#[derive(PartialEq)]
struct LogStamp {
    len: u64,
    modified: Option<SystemTime>,
    busy: bool,
}

type RunSummaries = BTreeMap<String, (LogStamp, HistoryIndexEntry)>;

static SUMMARIES: Mutex<BTreeMap<PathBuf, RunSummaries>> = Mutex::new(BTreeMap::new());

/// Returns the cached entry while the run's log is unchanged, otherwise calls `refresh`.
pub(super) fn cached_entry(
    history_root: &Path,
    entry: &HistoryIndexEntry,
    refresh: impl FnOnce(&HistoryIndexEntry) -> HistoryIndexEntry,
) -> HistoryIndexEntry {
    let Some(stamp) = log_stamp(entry) else {
        return refresh(entry);
    };
    let key = cache_key(entry);
    let cached = with_summaries(history_root, |runs| {
        runs.get(&key)
            .filter(|(cached_stamp, _)| *cached_stamp == stamp)
            .map(|(_, cached)| cached.clone())
    });
    cached.unwrap_or_else(|| {
        let refreshed = refresh(entry);
        with_summaries(history_root, |runs| {
            runs.insert(key, (stamp, refreshed.clone()))
        });
        refreshed
    })
}

/// Drops cached runs that left the index.
pub(super) fn keep_only(history_root: &Path, entries: &[HistoryIndexEntry]) {
    let keys: Vec<String> = entries.iter().map(cache_key).collect();
    with_summaries(history_root, |runs| {
        runs.retain(|key, _| keys.contains(key))
    });
}

fn cache_key(entry: &HistoryIndexEntry) -> String {
    format!("{}|{}", entry.run_id, entry.history_file_path)
}

fn log_stamp(entry: &HistoryIndexEntry) -> Option<LogStamp> {
    let metadata = fs::metadata(&entry.history_file_path).ok()?;
    Some(LogStamp {
        len: metadata.len(),
        modified: metadata.modified().ok(),
        busy: run_lease::is_busy(&entry.run_id),
    })
}

fn with_summaries<T>(history_root: &Path, use_runs: impl FnOnce(&mut RunSummaries) -> T) -> T {
    let mut roots = SUMMARIES.lock().unwrap_or_else(PoisonError::into_inner);
    use_runs(roots.entry(history_root.to_path_buf()).or_default())
}
