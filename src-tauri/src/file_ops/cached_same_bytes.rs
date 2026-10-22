use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex, MutexGuard, PoisonError},
    time::SystemTime,
};

use super::files_have_same_bytes;

/// Caps memory; a full cache starts over instead of tracking which entry is oldest.
const MAX_COMPARISONS: usize = 10_000;

#[derive(Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    len: u64,
    modified: Option<SystemTime>,
}

impl FileStamp {
    fn read(path: &Path) -> io::Result<Self> {
        let metadata = path.metadata()?;
        Ok(Self {
            len: metadata.len(),
            modified: metadata.modified().ok(),
        })
    }
}

struct Comparison {
    left: FileStamp,
    right: FileStamp,
    same: bool,
}

type Comparisons = HashMap<(PathBuf, PathBuf), Comparison>;

static COMPARISONS: LazyLock<Mutex<Comparisons>> = LazyLock::new(Mutex::default);

/// `files_have_same_bytes` that remembers the answer while neither file changes size or
/// modified time. Planning reruns on every setting change and would re-read whole videos.
pub(crate) fn files_have_same_bytes_cached(left: &Path, right: &Path) -> io::Result<bool> {
    let left_stamp = FileStamp::read(left)?;
    let right_stamp = FileStamp::read(right)?;
    if left_stamp.len != right_stamp.len {
        return Ok(false);
    }

    let key = (left.to_path_buf(), right.to_path_buf());
    if let Some(known) = comparisons().get(&key) {
        if known.left == left_stamp && known.right == right_stamp {
            return Ok(known.same);
        }
    }

    let same = files_have_same_bytes(left, right)?;
    let mut cache = comparisons();
    if cache.len() >= MAX_COMPARISONS {
        cache.clear();
    }
    cache.insert(
        key,
        Comparison {
            left: left_stamp,
            right: right_stamp,
            same,
        },
    );
    Ok(same)
}

fn comparisons() -> MutexGuard<'static, Comparisons> {
    COMPARISONS.lock().unwrap_or_else(PoisonError::into_inner)
}
