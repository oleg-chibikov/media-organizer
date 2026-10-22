use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{
    dated, photo_record,
    place_fixtures::{group, suggestion},
};
use crate::{
    events,
    models::{PlaceSuggestion, SuggestionKind},
    planner::{stream_move_plan, PlaceFolders},
    test_support::{events::RecordingSink, media::dated_metadata},
};

const DAY: &str = "2025-03-01 10:00:00";

fn abroad(kind: SuggestionKind, title: &str, place: &str, country: &str) -> PlaceSuggestion {
    let mut found = suggestion(kind, title, vec![group(place, DAY, DAY, &["a"])]);
    found.country = Some(country.to_owned());
    found
}

/// The plan of file `a`, taken on `DAY`.
fn plan_of(found: PlaceSuggestion) -> Value {
    let metadata = [dated_metadata("a", DAY)];
    let folders = PlaceFolders::new(&[found], &metadata).expect("valid suggestions");
    let sink = RecordingSink::default();
    let layout = dated(Path::new("Root"));
    stream_move_plan(
        &sink,
        &layout,
        &[photo_record("a", "jpg")],
        &metadata,
        &folders,
    );
    sink.payloads(events::PLAN_FILE)[0]["plan"].clone()
}

fn dir(plan: &Value) -> PathBuf {
    PathBuf::from(plan["destination_dir"].as_str().expect("a dir"))
}

#[test]
fn an_outing_abroad_sits_in_its_country_folder() {
    let plan = plan_of(abroad(SuggestionKind::Outing, "Zoo", "Zoo", "Australia"));

    assert_eq!(
        dir(&plan),
        Path::new("Root").join("2025").join("Australia").join("Zoo")
    );
    assert_eq!(plan["placement"]["country"], "Australia");
    assert_eq!(plan["placement"]["title"], "Zoo");
}

#[test]
fn a_trip_abroad_is_filed_under_its_country_whatever_its_title() {
    let plan = plan_of(abroad(SuggestionKind::Trip, "Holiday", "Yosemite", "USA"));

    assert_eq!(
        dir(&plan),
        Path::new("Root").join("2025").join("USA").join("Yosemite")
    );
    assert_eq!(plan["placement"]["title"], "USA");
}
