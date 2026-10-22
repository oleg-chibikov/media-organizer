use std::path::{Path, PathBuf};

use super::dated;
use crate::{
    events,
    models::{FileKind, FileRecord},
    planner::{placed_files_under, stream_move_plan, DestinationLayout, PlaceFolders},
    test_support::{
        events::RecordingSink,
        media::{dated_metadata, file_record},
        month_library::{MonthLibrary, USA},
    },
};

fn photo_at(path: &Path) -> FileRecord {
    let id = path.to_string_lossy().to_string();
    let stem = path.file_stem().expect("a stem").to_string_lossy();
    FileRecord {
        source_path: id.clone(),
        original_name: stem.to_string(),
        ..file_record(&id, "jpg", FileKind::Photo)
    }
}

/// Plans the placed `a.jpg` and a loose `new.jpg` of a May rescan with no suggestions:
/// `(destination dir, filename, placement title)` for each.
fn plan_rescan(
    library: &MonthLibrary,
    layout: &DestinationLayout,
) -> Vec<(PathBuf, String, Option<String>)> {
    let may = library.month("5");
    let placed = may.join(USA).join("Yosemite").join("05-03").join("a.jpg");
    let loose = may.join("3").join("new.jpg");
    let files = [photo_at(&placed), photo_at(&loose)];
    let metadata = [
        dated_metadata(&files[0].id, "2026-05-03 10:00:00"),
        dated_metadata(&files[1].id, "2026-05-03 11:00:00"),
    ];
    let folders = PlaceFolders::default().with_placed(placed_files_under(
        &library.settings,
        &library.store,
        &may,
        &files,
    ));
    let sink = RecordingSink::default();
    stream_move_plan(&sink, 0, layout, &files, &metadata, &folders);
    sink.payloads(events::PLAN_FILE)
        .iter()
        .map(|payload| {
            let plan = &payload["plan"];
            (
                PathBuf::from(plan["destination_dir"].as_str().expect("a dir")),
                plan["destination_filename"]
                    .as_str()
                    .expect("a name")
                    .to_owned(),
                plan["placement"]["title"].as_str().map(str::to_owned),
            )
        })
        .collect()
}

#[test]
fn a_placed_file_stays_in_its_place_folder_in_date_folder_mode() {
    let library = MonthLibrary::new();
    library.place_may_usa_trip();
    let may = library.root().join("2026").join("5");

    let plans = plan_rescan(&library, &dated(&library.root()));

    let yosemite_day = may.join(USA).join("Yosemite").join("05-03");
    let renamed = "2026-05-03 10-00-00 (a) - iPhone 15 Pro.jpg".to_owned();
    assert_eq!(plans[0], (yosemite_day, renamed, Some(USA.to_owned())));
    assert_eq!(plans[1].0, may.join("3"));
    assert_eq!(plans[1].2, None);
}

#[test]
fn a_placed_file_keeps_its_placement_when_renamed_in_place() {
    let library = MonthLibrary::new();
    library.place_may_usa_trip();
    let layout = DestinationLayout::SameFolder { place_root: None };

    let plans = plan_rescan(&library, &layout);

    let yosemite_day = library.month("5").join(USA).join("Yosemite").join("05-03");
    assert_eq!(plans[0].0, yosemite_day);
    assert_eq!(plans[0].2.as_deref(), Some(USA));
}
