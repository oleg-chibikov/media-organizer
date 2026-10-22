use super::session_in;
use crate::{
    metadata_queue::metadata_error_result,
    models::FileKind,
    scan_session::{CurrentScan, PlanContext},
    test_support::media::file_record,
};

#[test]
fn looks_files_up_in_the_order_asked() {
    let temp = tempfile::tempdir().expect("temp");
    let session = session_in(temp.path(), temp.path());
    session.add_records(&[
        file_record("a", "jpg", FileKind::Photo),
        file_record("b", "jpg", FileKind::Photo),
    ]);
    session.add_metadata(&[metadata_error_result("b".to_string(), "x".to_string())]);

    let ids = ["b".to_string(), "a".to_string()];
    let records = session.records(&ids).expect("records");
    assert_eq!(records[0].id, "b");
    assert_eq!(records[1].id, "a");
    assert_eq!(
        session.metadata(&ids).len(),
        1,
        "unread files have no date yet"
    );
    assert!(session.records(&["c".to_string()]).is_err());
}

#[test]
fn a_new_scan_replaces_the_old_session() {
    let temp = tempfile::tempdir().expect("temp");
    let context = PlanContext {
        folder: temp.path().to_path_buf(),
        settings: crate::settings::SettingsStore::in_app_data(temp.path()),
        placements: crate::placements::PlacementStore::in_app_data(temp.path()),
    };
    let current = CurrentScan::default();
    current.begin(1, context.clone());
    current.begin(2, context);
    assert!(current.session(1).is_err());
    assert!(current.session(2).is_ok());
}
