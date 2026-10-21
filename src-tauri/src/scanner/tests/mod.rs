use std::{path::Path, sync::mpsc, sync::Arc, thread, time::Duration};

use crate::models::FileKind;
use crate::scanner::classification::{classify_extension, is_in_processed};

#[test]
fn classifies_supported_extensions() {
    assert!(matches!(classify_extension("jpg"), FileKind::Photo));
    assert!(matches!(classify_extension("HEIC"), FileKind::Photo));
    assert!(matches!(classify_extension("m2ts"), FileKind::Video));
}

#[test]
fn classifies_unsupported_and_unknown_extensions() {
    assert!(matches!(classify_extension("xmp"), FileKind::Unsupported));
    assert!(matches!(classify_extension(""), FileKind::Unknown));
    assert!(matches!(classify_extension("raw"), FileKind::Unknown));
}

#[test]
fn detects_processed_segment_in_relative_path() {
    let root = Path::new("todo");
    assert!(is_in_processed(
        root,
        &root
            .join("Processed")
            .join("2026")
            .join("01")
            .join("example.jpg")
    ));
    assert!(is_in_processed(
        root,
        &root.join("nested").join("processed").join("image.jpg")
    ));
    assert!(!is_in_processed(
        root,
        &root.join("nested").join("inbox").join("image.jpg")
    ));
}

#[test]
fn starting_a_scan_cancels_the_previous_one() {
    let active_scan = crate::scanner::ActiveScan::default();
    let older = active_scan.begin();
    let newer = active_scan.begin();
    assert!(older.token.is_cancelled());
    assert!(!newer.token.is_cancelled());

    active_scan.finish(&older);
    active_scan.cancel();

    assert!(newer.token.is_cancelled());
}

#[test]
fn a_new_scan_starts_only_after_the_previous_one_finished() {
    let active_scan = Arc::new(crate::scanner::ActiveScan::default());
    let (order_tx, order_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let older = active_scan.begin();
    let older_scan = {
        let (active_scan, order_tx) = (active_scan.clone(), order_tx.clone());
        thread::spawn(move || {
            active_scan.run(&older, |token| {
                order_tx.send("older started").expect("send");
                release_rx.recv().expect("release");
                order_tx
                    .send(if token.is_cancelled() {
                        "older cancelled"
                    } else {
                        "older ran on"
                    })
                    .expect("send");
            });
        })
    };
    assert_eq!(order_rx.recv().expect("older start"), "older started");

    let newer = active_scan.begin();
    let newer_scan = thread::spawn(move || {
        active_scan.run(&newer, |_| order_tx.send("newer started").expect("send"));
    });
    assert!(order_rx.recv_timeout(Duration::from_millis(50)).is_err());
    release_tx.send(()).expect("release older scan");
    older_scan.join().expect("older scan");
    newer_scan.join().expect("newer scan");

    let order: Vec<&str> = order_rx.try_iter().collect();
    assert_eq!(order, ["older cancelled", "newer started"]);
}

#[test]
fn streams_discovered_and_skipped_files_then_totals() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("Processed")).expect("create Processed");
    std::fs::write(root.join("photo.jpg"), b"photo").expect("write photo");
    std::fs::write(root.join("notes.txt"), b"notes").expect("write notes");
    std::fs::write(root.join("Processed").join("done.jpg"), b"done").expect("write done");
    let sink = crate::test_support::events::RecordingSink::default();

    crate::scanner::scan_folder(&sink, root, &tokio_util::sync::CancellationToken::new());

    let names = sink.names();
    assert_eq!(names.first(), Some(&crate::events::SCAN_STARTED));
    assert_eq!(names.last(), Some(&crate::events::SCAN_FINISHED));
    let finished = &sink.payloads(crate::events::SCAN_FINISHED)[0];
    assert_eq!(finished["discovered"], 1);
    assert_eq!(finished["skipped"], 2);
    assert_eq!(finished["cancelled"], false);
}

mod misnamed_media;
