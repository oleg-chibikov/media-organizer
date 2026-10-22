use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use tokio_util::sync::CancellationToken;

use super::{start_reading, BatchReader, MetadataPause, ReadOptions};
use crate::{
    events::{METADATA_FINISHED, METADATA_RESULTS},
    metadata_queue::metadata_error_result,
    models::{FileKind, FileRecord, MetadataReadRequest, MetadataResult, ScanStatus},
    test_support::events::RecordingSink,
};

/// Answers every request with an error result that names the file, and counts the reads.
#[derive(Default)]
struct FakeReader {
    reads: Mutex<Vec<String>>,
}

impl BatchReader for FakeReader {
    fn read(&self, requests: Vec<MetadataReadRequest>) -> Vec<MetadataResult> {
        let mut reads = self.reads.lock().expect("reads lock");
        requests
            .into_iter()
            .map(|request| {
                reads.push(request.file_id.clone());
                metadata_error_result(request.file_id, "fake".to_string())
            })
            .collect()
    }
}

fn record(id: &str, kind: FileKind) -> FileRecord {
    FileRecord {
        id: id.to_string(),
        source_path: format!("/in/{id}"),
        relative_path: id.to_string(),
        extension: "jpg".to_string(),
        original_name: id.to_string(),
        kind,
        scan_status: ScanStatus::Discovered,
    }
}

fn options(reader: &Arc<FakeReader>, token: &CancellationToken) -> ReadOptions<FakeReader> {
    ReadOptions {
        scan_id: 3,
        token: token.clone(),
        reader: reader.clone(),
        pause: Arc::new(MetadataPause::default()),
        reader_count: 2,
    }
}

fn wait_for_finish(sink: &RecordingSink) -> serde_json::Value {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Some(finished) = sink.payloads(METADATA_FINISHED).pop() {
            return finished;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("the readers did not finish");
}

#[test]
fn reads_photos_and_videos_then_reports_the_scan_finished() {
    let sink = Arc::new(RecordingSink::default());
    let reader = Arc::new(FakeReader::default());
    let token = CancellationToken::new();
    let feed = start_reading(sink.clone(), options(&reader, &token));
    let records: Vec<FileRecord> = (0..20)
        .map(|index| record(&format!("p{index}"), FileKind::Photo))
        .chain([
            record("v", FileKind::Video),
            record("t", FileKind::Unsupported),
        ])
        .collect();
    feed.push(&records);
    drop(feed);

    let finished = wait_for_finish(&sink);
    assert_eq!(finished["scan_id"], 3);
    assert_eq!(finished["cancelled"], false);
    let read: usize = sink
        .payloads(METADATA_RESULTS)
        .iter()
        .map(|event| event["results"].as_array().map_or(0, Vec::len))
        .sum();
    assert_eq!(
        read, 21,
        "every photo and video gets one result, other files none"
    );
    assert_eq!(sink.names().last(), Some(&METADATA_FINISHED));
}

#[test]
fn a_cancelled_scan_stops_reading_and_sends_no_more_dates() {
    let sink = Arc::new(RecordingSink::default());
    let reader = Arc::new(FakeReader::default());
    let token = CancellationToken::new();
    let opts = options(&reader, &token);
    opts.pause.set(true);
    let feed = start_reading(sink.clone(), opts);
    feed.push(&[record("a", FileKind::Photo)]);
    token.cancel();
    drop(feed);

    let finished = wait_for_finish(&sink);
    assert_eq!(finished["cancelled"], true);
    assert!(sink.payloads(METADATA_RESULTS).is_empty());
    assert!(reader.reads.lock().expect("reads lock").is_empty());
}

#[test]
fn a_paused_reader_waits_for_the_resume() {
    let sink = Arc::new(RecordingSink::default());
    let reader = Arc::new(FakeReader::default());
    let token = CancellationToken::new();
    let opts = options(&reader, &token);
    let pause = opts.pause.clone();
    pause.set(true);
    let feed = start_reading(sink.clone(), opts);
    feed.push(&[record("a", FileKind::Photo)]);
    drop(feed);
    std::thread::sleep(Duration::from_millis(50));
    assert!(reader.reads.lock().expect("reads lock").is_empty());

    pause.set(false);
    assert_eq!(wait_for_finish(&sink)["cancelled"], false);
    assert_eq!(
        *reader.reads.lock().expect("reads lock"),
        vec!["a".to_string()]
    );
}
