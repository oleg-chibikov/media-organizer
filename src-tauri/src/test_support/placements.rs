use std::{fs, path::Path};

use chrono::NaiveDateTime;

use crate::{
    models::{PlaceGroup, PlaceSuggestion, Placement, SuggestionKind},
    placements::{record_moves, MovedIntoPlace, PlacementStore},
    trips::{day_layout_for_span, CHOSEN_DATE_FORMAT},
};

pub(crate) fn placement(
    kind: SuggestionKind,
    year: i32,
    title: &str,
    place: Option<&str>,
) -> Placement {
    Placement {
        kind,
        year,
        title: title.to_owned(),
        place: place.map(str::to_owned),
    }
}

/// A file moved from outside `root` to `root/<relative>`.
pub(crate) fn moved_into(
    root: &Path,
    placement: &Placement,
    relative: &str,
    date: &str,
) -> MovedIntoPlace {
    MovedIntoPlace {
        placement: placement.clone(),
        source: Path::new("/elsewhere/todo").join(relative),
        destination: root.join(relative),
        chosen_date: date.to_owned(),
    }
}

/// Writes each moved file under `root` and records the moves in `store`.
pub(crate) fn place_on_disk(store: &PlacementStore, root: &Path, moves: &[MovedIntoPlace]) {
    for moved in moves {
        fs::create_dir_all(moved.destination.parent().expect("a parent")).expect("create dir");
        fs::write(&moved.destination, moved.chosen_date.as_bytes()).expect("write file");
    }
    store
        .update(|records| record_moves(records, root, moves))
        .expect("record moves");
}

pub(crate) fn place_group(name: &str, start: &str, end: &str, file_ids: &[&str]) -> PlaceGroup {
    let date = |value: &str| {
        NaiveDateTime::parse_from_str(value, CHOSEN_DATE_FORMAT)
            .expect("a date")
            .date()
    };
    PlaceGroup {
        name: name.to_owned(),
        place_kind: "city".to_owned(),
        day_layout: day_layout_for_span(date(start), date(end)),
        start: start.to_owned(),
        end: end.to_owned(),
        file_ids: file_ids.iter().map(|id| (*id).to_owned()).collect(),
    }
}

/// A suggestion over its places, as detection makes it.
pub(crate) fn detected(
    kind: SuggestionKind,
    title: &str,
    places: Vec<PlaceGroup>,
) -> PlaceSuggestion {
    let start = places.iter().map(|place| place.start.clone()).min();
    let end = places.iter().map(|place| place.end.clone()).max();
    let start = start.expect("a place");
    PlaceSuggestion {
        id: format!("{title}-{start}"),
        kind,
        year: start[..4].parse().expect("a year"),
        title: title.to_owned(),
        start,
        end: end.expect("a place"),
        file_count: 0,
        places,
        root_file_ids: Vec::new(),
        continues: None,
    }
}
