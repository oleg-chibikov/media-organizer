use std::path::Path;

use crate::{
    events,
    models::FileKind,
    planner::stream_move_plan,
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

    stream_move_plan(&sink, Path::new("MediaRoot"), &[photo, notes], &[metadata]);

    assert_eq!(sink.names(), vec![events::PLAN_FILE, events::PLAN_FINISHED]);
    let finished = &sink.payloads(events::PLAN_FINISHED)[0];
    assert_eq!(finished["planned"], 1);
    assert_eq!(finished["errors"], 0);
}
