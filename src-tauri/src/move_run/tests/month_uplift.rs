use std::path::Path;

use tokio_util::sync::CancellationToken;

use crate::{
    history::HistoryStore,
    models::{PlaceSuggestion, SuggestionKind},
    move_run::{start_move_run, MoveBatch},
    placements::PlacedSuggestion,
    planner::plan_continued_moves,
    test_support::{
        events::RecordingSink,
        month_library::{MonthLibrary, USA},
        placements::{detected, place_group},
    },
    undo_redo::{prepare_replay, ReplayDirection},
};

fn lifted_trip(library: &MonthLibrary) -> PlaceSuggestion {
    let boston = place_group("Boston", "2026-04-02 10:00:00", "2026-04-02 10:00:00", &[]);
    let mut trip = detected(SuggestionKind::Trip, USA, vec![boston]);
    trip.country = Some(USA.to_owned());
    library.detect(&library.month("4"), vec![trip]).remove(0)
}

/// `(month folder, relative paths)` of each record, in order.
fn records(library: &MonthLibrary) -> Vec<(Option<String>, Vec<String>)> {
    let to_paths = |record: &PlacedSuggestion| {
        let places = record.places.iter().flat_map(|place| &place.files);
        let mut paths: Vec<String> = places
            .chain(&record.root_files)
            .map(|file| file.relative_path.clone())
            .collect();
        paths.sort();
        paths
    };
    let loaded = library.store.load().expect("registry");
    loaded
        .iter()
        .map(|record| (record.month_folder.clone(), to_paths(record)))
        .collect()
}

#[test]
fn a_lift_moves_the_month_folder_up_and_undo_puts_it_back() {
    let library = MonthLibrary::new();
    library.place_may_usa_trip();
    let may = library.month("5");
    let april = library.month("4");
    let trip = lifted_trip(&library);
    let moves = plan_continued_moves(&library.settings, &library.store, &april, &[trip], &[])
        .expect("planned");
    let batch = MoveBatch {
        files: moves.iter().map(|placed| placed.record.clone()).collect(),
        metadata_results: moves.iter().map(|placed| placed.metadata.clone()).collect(),
        plans: moves.iter().map(|placed| placed.plan.clone()).collect(),
    };
    let (sink, app_data) = (RecordingSink::default(), library.app_data());
    let history = HistoryStore::in_app_data(&app_data);

    start_move_run(
        &sink,
        &library.settings,
        &history,
        &library.store,
        &april,
        batch,
    )
    .expect("run starts")
    .execute(&sink, &CancellationToken::new());

    let usa = library.root().join("2026").join(USA);
    assert!(usa.join("Yosemite").join("05-03").join("a.jpg").exists());
    assert!(usa.join("5").join("5").join("r.jpg").exists());
    assert!(may.is_dir() && !may.join(USA).exists());
    let lifted = vec![
        "2026/USA/5/5/r.jpg".to_owned(),
        "2026/USA/Yosemite/05-03/a.jpg".to_owned(),
        "2026/USA/Yosemite/05-04/b.jpg".to_owned(),
    ];
    assert_eq!(records(&library), [(None, lifted)]);

    let run_id = history.list_runs().expect("runs")[0].run_id.clone();
    prepare_replay(&history, &library.store, &run_id, ReplayDirection::Undo)
        .expect("undoable")
        .run(&sink);

    let back = Path::new(&may).join(USA);
    assert!(back.join("Yosemite").join("05-03").join("a.jpg").exists());
    assert!(back.join("05-05").join("r.jpg").exists());
    let restored = vec![
        "2026/5/USA/05-05/r.jpg".to_owned(),
        "2026/5/USA/Yosemite/05-03/a.jpg".to_owned(),
        "2026/5/USA/Yosemite/05-04/b.jpg".to_owned(),
    ];
    assert_eq!(records(&library), [(Some("5".to_owned()), restored)]);
}
