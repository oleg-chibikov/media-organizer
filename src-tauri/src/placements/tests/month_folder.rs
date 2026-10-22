use std::path::Path;

use crate::placements::MonthFolder;

const ROOT: &str = "/Library";

fn month_of(selected: &str) -> Option<MonthFolder> {
    MonthFolder::of(Path::new(ROOT), &Path::new(ROOT).join(selected))
}

fn month(year: i32, name: &str) -> Option<MonthFolder> {
    Some(MonthFolder {
        year,
        name: name.to_owned(),
    })
}

#[test]
fn a_month_folder_keeps_its_name_as_on_disk() {
    assert_eq!(month_of("2026/5"), month(2026, "5"));
    assert_eq!(month_of("2026/05"), month(2026, "05"));
}

#[test]
fn a_day_folder_belongs_to_its_month() {
    assert_eq!(month_of("2026/5/12"), month(2026, "5"));
}

#[test]
fn other_folders_are_not_months() {
    for selected in [
        "",
        "2026",
        "2026/13",
        "2026/0",
        "2026/USA",
        "26/5",
        "2026/5/12/x",
        "2026/5/32",
    ] {
        assert_eq!(month_of(selected), None, "{selected}");
    }
    let camera = MonthFolder::of(Path::new(ROOT), Path::new("/Camera/2026/5"));
    assert_eq!(camera, None);
}
