use std::collections::HashMap;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use super::{photo_in, session_in};
use crate::{
    events::{MOVES_DONE, MOVE_FILE},
    history::HistoryStore,
    test_support::{events::RecordingSink, media::dated_metadata},
};

const DATE: &str = "2026-03-05 07:20:17";

fn wait_for(sink: &RecordingSink, event: &str) -> Option<serde_json::Value> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Some(payload) = sink.payloads(event).pop() {
            return Some(payload);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    None
}

#[test]
fn moves_files_as_they_become_ready_then_cleans_the_source() {
    let todo = tempfile::tempdir().expect("todo");
    let app_data = tempfile::tempdir().expect("app data");
    let session = session_in(todo.path(), app_data.path());
    let sink = Arc::new(RecordingSink::default());
    let photo = photo_in(todo.path(), "a", "IMG_1");
    session.add_records(std::slice::from_ref(&photo));
    let dates = [dated_metadata(&photo.id, DATE)];
    session.add_metadata(&dates);

    let history = HistoryStore::in_app_data(app_data.path());
    session
        .start_moves(sink.clone(), history)
        .expect("moves start");
    assert!(session
        .start_moves(sink.clone(), HistoryStore::in_app_data(app_data.path()))
        .is_err());
    std::thread::sleep(Duration::from_millis(50));
    assert!(
        sink.payloads(MOVE_FILE).is_empty(),
        "plans wait for the place folders"
    );

    session.set_places(&sink, HashMap::new(), true);
    std::thread::sleep(Duration::from_millis(50));
    assert!(
        sink.payloads(MOVE_FILE).is_empty(),
        "plans of a preview don't move"
    );

    session.set_places(&sink, HashMap::new(), false);
    session.mark_scanned();
    session.mark_dates_read();

    let done = wait_for(&sink, MOVES_DONE).expect("moves done");
    assert_eq!(done["error"], serde_json::Value::Null);
    assert_eq!(sink.payloads(MOVE_FILE).len(), 1);
    assert!(
        !todo.path().join("a").exists(),
        "the emptied source folder goes"
    );
}

#[test]
fn a_paused_run_waits_for_the_resume() {
    let todo = tempfile::tempdir().expect("todo");
    let app_data = tempfile::tempdir().expect("app data");
    let session = session_in(todo.path(), app_data.path());
    let sink = Arc::new(RecordingSink::default());
    let photo = photo_in(todo.path(), "a", "IMG_1");
    session.add_records(std::slice::from_ref(&photo));
    session.add_metadata(&[dated_metadata(&photo.id, DATE)]);
    session.set_places(&sink, HashMap::new(), false);
    session.mark_scanned();
    session.mark_dates_read();

    session.pause_moves();
    let history = HistoryStore::in_app_data(app_data.path());
    session
        .start_moves(sink.clone(), history)
        .expect("moves start");
    std::thread::sleep(Duration::from_millis(50));
    assert!(sink.payloads(MOVE_FILE).is_empty());

    session.resume_moves();
    assert!(wait_for(&sink, MOVES_DONE).is_some());
    assert_eq!(sink.payloads(MOVE_FILE).len(), 1);
}
