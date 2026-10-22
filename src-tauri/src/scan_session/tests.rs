use crate::{
    metadata_queue::metadata_error_result,
    models::{FileKind, FileRecord, ScanStatus},
};

use super::CurrentScan;

fn record(id: &str) -> FileRecord {
    FileRecord {
        id: id.to_string(),
        source_path: format!("/in/{id}.jpg"),
        relative_path: format!("{id}.jpg"),
        extension: "jpg".to_string(),
        original_name: format!("{id}.jpg"),
        kind: FileKind::Photo,
        scan_status: ScanStatus::Discovered,
    }
}

#[test]
fn looks_files_up_in_the_order_asked() {
    let current = CurrentScan::default();
    let session = current.begin(4);
    session.add_records(&[record("a"), record("b")]);
    session.add_metadata(&[metadata_error_result("b".to_string(), "x".to_string())]);

    let ids = ["b".to_string(), "a".to_string()];
    let records = current
        .session(4)
        .and_then(|s| s.records(&ids))
        .expect("records");
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
    let current = CurrentScan::default();
    current.begin(1);
    current.begin(2);
    assert!(current.session(1).is_err());
    assert!(current.session(2).is_ok());
}
