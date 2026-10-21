use tempfile::tempdir;

use crate::{
    history::HistoryStore,
    models::{DestinationRootMode, EffectiveDestinationRoot, HistoryRunStatus},
};

#[test]
fn lists_a_run_in_progress_as_running_until_its_job_stops() {
    let temp = tempdir().expect("temp dir");
    let store = HistoryStore::in_app_data(temp.path());
    let root = EffectiveDestinationRoot {
        mode: DestinationRootMode::ProcessedFolder,
        root_path: temp.path().join("Processed").to_string_lossy().to_string(),
        custom_root_configured: false,
        force_processed_folder: false,
        warning: None,
    };
    let run = store.start_run(temp.path(), &root).expect("start run");

    let listed = store.list_runs().expect("list runs");
    assert_eq!(listed[0].run_status, HistoryRunStatus::Running);
    assert!(!listed[0].undo_available);

    // A run whose job stopped without a completion record was cut short.
    drop(run);
    let listed = store.list_runs().expect("list runs");
    assert_eq!(listed[0].run_status, HistoryRunStatus::Partial);
}
