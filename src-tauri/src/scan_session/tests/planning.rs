use std::collections::HashMap;

use super::{photo_in, session_in};
use crate::{
    events::{PLAN_CLEARED, PLAN_FILES, PLAN_SETTLED},
    test_support::{events::RecordingSink, media::dated_metadata},
};

const DATE: &str = "2026-03-05 07:20:17";

#[test]
fn waits_for_the_place_folders_then_plans_every_dated_file() {
    let temp = tempfile::tempdir().expect("temp");
    let app_data = tempfile::tempdir().expect("app data");
    let session = session_in(temp.path(), app_data.path());
    let sink = RecordingSink::default();
    let photo = photo_in(temp.path(), "a", "IMG_1");
    session.add_records(std::slice::from_ref(&photo));
    let dates = [dated_metadata(&photo.id, DATE)];
    session.add_metadata(&dates);

    session.plan_read(&sink, &dates);
    assert!(
        sink.payloads(PLAN_FILES).is_empty(),
        "plans wait for the place folders"
    );

    session.set_places(&sink, HashMap::new(), false);
    assert_eq!(sink.payloads(PLAN_FILES).len(), 1);
    assert_eq!(sink.names().last(), Some(&PLAN_SETTLED));
    assert!(session.plans(std::slice::from_ref(&photo.id)).is_ok());

    session.set_places(&sink, HashMap::new(), false);
    assert_eq!(
        sink.payloads(PLAN_FILES).len(),
        1,
        "an unchanged file isn't planned again"
    );
}

#[test]
fn files_planned_apart_never_share_a_destination() {
    let temp = tempfile::tempdir().expect("temp");
    let app_data = tempfile::tempdir().expect("app data");
    let session = session_in(temp.path(), app_data.path());
    let sink = RecordingSink::default();
    session.set_places(&sink, HashMap::new(), false);
    let first = photo_in(temp.path(), "a", "IMG_1");
    let second = photo_in(temp.path(), "b", "IMG_1");

    for photo in [&first, &second] {
        session.add_records(std::slice::from_ref(photo));
        let dates = [dated_metadata(&photo.id, DATE)];
        session.add_metadata(&dates);
        session.plan_read(&sink, &dates);
    }

    let plans = session
        .plans(&[first.id.clone(), second.id.clone()])
        .expect("plans");
    assert_ne!(plans[0].destination_path, plans[1].destination_path);
}

#[test]
fn clearing_drops_every_plan_until_places_apply_again() {
    let temp = tempfile::tempdir().expect("temp");
    let app_data = tempfile::tempdir().expect("app data");
    let session = session_in(temp.path(), app_data.path());
    let sink = RecordingSink::default();
    let photo = photo_in(temp.path(), "a", "IMG_1");
    session.add_records(std::slice::from_ref(&photo));
    let dates = [dated_metadata(&photo.id, DATE)];
    session.add_metadata(&dates);
    session.set_places(&sink, HashMap::new(), false);

    session.clear_plans(&sink);
    assert_eq!(sink.names().last(), Some(&PLAN_CLEARED));
    assert!(session.plans(std::slice::from_ref(&photo.id)).is_err());
    session.plan_read(&sink, &dates);
    assert_eq!(
        sink.payloads(PLAN_FILES).len(),
        1,
        "no plan until places apply again"
    );

    session.set_places(&sink, HashMap::new(), false);
    assert_eq!(sink.payloads(PLAN_FILES).len(), 2);
}
