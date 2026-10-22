use std::path::Path;

use super::dated;
use crate::{
    events,
    models::FileKind,
    planner::{stream_move_plan, PlaceFolders},
    test_support::{
        events::RecordingSink,
        media::{dated_metadata, file_record},
    },
};

#[test]
fn streams_one_plan_per_media_file_then_totals() {
    let photo = file_record("IMG_1001", "jpg", FileKind::Photo);
    let notes = file_record("notes", "txt", FileKind::Unsupported);
    let metadata = dated_metadata("IMG_1001", "2026-03-05 07:20:17");
    let sink = RecordingSink::default();

    stream_move_plan(
        &sink,
        7,
        &dated(Path::new("MediaRoot")),
        &[photo, notes],
        &[metadata],
        &PlaceFolders::default(),
    );

    assert_eq!(sink.names(), vec![events::PLAN_FILE, events::PLAN_FINISHED]);
    let finished = &sink.payloads(events::PLAN_FINISHED)[0];
    assert_eq!(finished["planned"], 1);
    assert_eq!(finished["errors"], 0);
    assert_eq!(finished["request_id"], 7);
    assert_eq!(sink.payloads(events::PLAN_FILE)[0]["request_id"], 7);
}
