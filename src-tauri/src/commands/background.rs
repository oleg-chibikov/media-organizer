//! Keeps filesystem work off the UI thread and the async runtime workers.

/// Runs `job` on the blocking pool and waits for its result.
pub(super) async fn run_blocking<T: Send + 'static>(
    job: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(job)
        .await
        .map_err(|error| format!("Background task failed: {error}"))?
}

/// Starts a long job that reports through events; the command returns at once.
pub(super) fn spawn_job(job: impl FnOnce() + Send + 'static) {
    drop(tauri::async_runtime::spawn_blocking(job));
}
