use std::time::Duration;

use crate::metadata;

const MIN_METADATA_CONCURRENCY: usize = 1;
const MAX_METADATA_CONCURRENCY: usize = 8;
const DEFAULT_METADATA_QUEUE_CAPACITY: usize = 256;
const MAX_METADATA_QUEUE_CAPACITY: usize = 4096;

/// Queue sizing, overridable through `MEDIA_ORGANIZER_METADATA_*` environment variables.
pub(super) struct QueueLimits {
    pub concurrency: usize,
    pub queue_capacity: usize,
    pub timeout: Duration,
}

impl QueueLimits {
    pub(super) fn from_env() -> Self {
        let concurrency = read_env_usize(
            "MEDIA_ORGANIZER_METADATA_CONCURRENCY",
            default_metadata_concurrency(),
            MIN_METADATA_CONCURRENCY,
            MAX_METADATA_CONCURRENCY,
        );
        let queue_capacity = read_env_usize(
            "MEDIA_ORGANIZER_METADATA_QUEUE_CAPACITY",
            DEFAULT_METADATA_QUEUE_CAPACITY,
            concurrency,
            MAX_METADATA_QUEUE_CAPACITY,
        );
        let timeout_ms = read_env_u64(
            "MEDIA_ORGANIZER_METADATA_TIMEOUT_MS",
            metadata::DEFAULT_REQUEST_TIMEOUT_MS,
            1_000,
            120_000,
        );

        Self {
            concurrency,
            queue_capacity,
            timeout: Duration::from_millis(timeout_ms),
        }
    }
}

fn default_metadata_concurrency() -> usize {
    std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .clamp(MIN_METADATA_CONCURRENCY, MAX_METADATA_CONCURRENCY)
}

fn read_env_usize(name: &str, default: usize, min: usize, max: usize) -> usize {
    match std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
    {
        Some(value) => value.clamp(min, max),
        None => default,
    }
}

fn read_env_u64(name: &str, default: u64, min: u64, max: u64) -> u64 {
    match std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
    {
        Some(value) => value.clamp(min, max),
        None => default,
    }
}
