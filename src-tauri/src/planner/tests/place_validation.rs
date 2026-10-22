use super::place_fixtures::{group, suggestion};
use crate::{
    models::{PlaceSuggestion, SuggestionKind},
    planner::PlaceFolders,
};

fn outing_named(name: &str) -> PlaceSuggestion {
    let place = group(name, "2025-03-01 10:00:00", "2025-03-01 11:00:00", &["a"]);
    suggestion(SuggestionKind::Outing, name, vec![place])
}

fn folders_for(suggestions: &[PlaceSuggestion]) -> Result<PlaceFolders, String> {
    PlaceFolders::new(suggestions, &[])
}

#[test]
fn an_empty_name_is_rejected() {
    let error = folders_for(&[outing_named("  ")]).expect_err("an empty name");
    assert!(error.contains("empty"), "{error}");
}

#[test]
fn path_like_names_are_rejected() {
    for name in ["../Photos", r"Trips\Italy", "..", "a/b"] {
        let error = folders_for(&[outing_named(name)]).expect_err(name);
        assert!(error.contains("looks like a path"), "{error}");
    }
}

#[test]
fn edited_names_get_the_windows_safe_form() {
    let folders = folders_for(&[outing_named(" Zoo: day out? ")]).expect("a valid name");
    let folder = folders.get("a").expect("a folder");
    assert_eq!(
        folder.dir_under(std::path::Path::new(""), &Default::default()),
        std::path::Path::new("2025").join("Zoo_ day out_")
    );
}

#[test]
fn an_outing_with_two_places_is_rejected() {
    let mut outing = outing_named("Zoo");
    outing.places.push(outing.places[0].clone());
    let error = folders_for(&[outing]).expect_err("two places");
    assert!(error.contains("exactly one place"), "{error}");
}

#[test]
fn a_file_in_two_suggestions_is_rejected() {
    let error = folders_for(&[outing_named("Zoo"), outing_named("Garden")]).expect_err("a twin");
    assert!(error.contains("more than one place folder"), "{error}");
}
