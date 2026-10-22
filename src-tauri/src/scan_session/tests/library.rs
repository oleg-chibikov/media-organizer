use std::{collections::HashMap, fs, sync::Arc};

use tempfile::{tempdir, TempDir};

use super::{moves::wait_for, photo_in, session_in};
use crate::{
    events::MOVES_DONE,
    history::HistoryStore,
    models::{FilePlace, FileRecord, HomeMatch},
    scan_session::ScanSession,
    settings::set_root_media_folder,
    test_support::{events::RecordingSink, media::dated_metadata},
};

fn dubai() -> FilePlace {
    FilePlace {
        place: "Dubai".to_owned(),
        country: "Australia".to_owned(),
        home: HomeMatch::Country,
    }
}

struct Dirs {
    root: TempDir,
    app_data: TempDir,
    todo: TempDir,
}

/// A scan of one Dubai photo on March 5, with a custom root that has Dubai on March 4.
fn dubai_scan_after_an_earlier_day() -> (Dirs, Arc<ScanSession>, FileRecord) {
    let dirs = Dirs {
        root: tempdir().expect("root"),
        app_data: tempdir().expect("app data"),
        todo: tempdir().expect("todo"),
    };
    let session = session_in(dirs.todo.path(), dirs.app_data.path());
    let root = dirs.root.path().to_string_lossy().to_string();
    session
        .context
        .settings
        .update(|settings| set_root_media_folder(settings, Some(root)))
        .expect("custom root");
    let old = dirs.root.path().join("2026/3/4/Dubai/old.jpg");
    fs::create_dir_all(old.parent().expect("parent")).expect("folder");
    fs::write(&old, b"old").expect("old photo");
    let photo = photo_in(dirs.todo.path(), "a", "IMG_1");
    session.add_records(std::slice::from_ref(&photo));
    session.add_metadata(&[dated_metadata(&photo.id, "2026-03-05 07:20:17")]);
    (dirs, session, photo)
}

#[test]
fn an_earlier_day_of_the_place_moves_with_the_run_and_its_folder_goes() {
    let (dirs, session, photo) = dubai_scan_after_an_earlier_day();
    let root = dirs.root.path();
    let sink = Arc::new(RecordingSink::default());

    session.set_places(&sink, HashMap::from([(photo.id.clone(), dubai())]), false);
    let (gather, folders) = session.library_folders();
    assert!(gather);
    assert_eq!(folders.len(), 1);
    assert_eq!(folders[0].files, 1);

    session.mark_scanned();
    session.mark_dates_read();
    let history = HistoryStore::in_app_data(dirs.app_data.path());
    session
        .start_moves(sink.clone(), history)
        .expect("moves start");
    wait_for(&sink, MOVES_DONE).expect("moves done");

    assert!(root.join("2026/3/Dubai/4/old.jpg").is_file());
    assert!(
        !root.join("2026/3/4").exists(),
        "the emptied day folder goes"
    );
    let new_day = fs::read_dir(root.join("2026/3/Dubai/5")).expect("new day");
    assert_eq!(new_day.count(), 1);
}

#[test]
fn leaving_earlier_days_apart_plans_the_day_folder_again() {
    let (_dirs, session, photo) = dubai_scan_after_an_earlier_day();
    let sink = RecordingSink::default();
    session.set_places(&sink, HashMap::from([(photo.id.clone(), dubai())]), false);

    session.set_gather_library(&sink, false);

    let (gather, folders) = session.library_folders();
    assert!(!gather);
    assert!(folders.is_empty());
    let plan = &session
        .plans(std::slice::from_ref(&photo.id))
        .expect("plan")[0];
    assert!(plan.destination_dir.ends_with("2026/3/5/Dubai"));
}
