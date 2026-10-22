use std::path::{Path, PathBuf};

use super::{dated, place, planned_dirs};
use crate::models::{FilePlace, HomeMatch};

#[test]
fn files_in_home_city_keep_date_folders_and_dont_change_other_levels() {
    let sydney = FilePlace {
        home: HomeMatch::City,
        ..place("Sydney", None)
    };
    let dirs = planned_dirs(
        &dated(Path::new("Media")),
        &[
            ("a", "2024-03-15 10:00:00", Some(sydney.clone())),
            ("b", "2024-06-01 10:00:00", Some(sydney)),
            ("c", "2024-03-20 10:00:00", Some(place("Melbourne", None))),
        ],
    );

    let path = |parts: &str| PathBuf::from("Media").join(parts);
    assert_eq!(
        dirs,
        [
            path("2024/3/15"),
            path("2024/6/1"),
            path("2024/3/20/Melbourne")
        ]
    );
}
