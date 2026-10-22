use std::{collections::BTreeSet, path::PathBuf};

use chrono::{Datelike, NaiveDate};

/// The date folder a country or place folder sits in. Higher levels sort first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum PlaceLevel {
    /// Files in more than one month: `<year>/<folder>/<month>/<day>`.
    Year,
    /// Files on more than one day of one month: `<year>/<month>/<folder>/<day>`.
    Month,
    /// Files on one day: `<year>/<month>/<day>/<folder>`.
    Day,
}

impl PlaceLevel {
    pub(super) const ALL: [Self; 3] = [Self::Year, Self::Month, Self::Day];
}

/// The days a folder's files were taken on, within one year.
#[derive(Debug, Default)]
pub(super) struct DaySpan(BTreeSet<(u32, u32)>);

impl DaySpan {
    pub(super) fn add(&mut self, date: NaiveDate) {
        self.0.insert((date.month(), date.day()));
    }

    pub(super) fn months(&self) -> BTreeSet<u32> {
        self.0.iter().map(|(month, _)| *month).collect()
    }

    pub(super) fn level(&self) -> PlaceLevel {
        if self.months().len() > 1 {
            PlaceLevel::Year
        } else if self.0.len() > 1 {
            PlaceLevel::Month
        } else {
            PlaceLevel::Day
        }
    }

    pub(super) fn first_date(&self, year: i32) -> Option<NaiveDate> {
        let &(month, day) = self.0.first()?;
        NaiveDate::from_ymd_opt(year, month, day)
    }
}

/// `<year>/<month>/<day>` with each folder after the date part of its level. With
/// `to_last_folder` the path stops at the deepest folder.
pub(super) fn dated_dir(
    date: NaiveDate,
    folders: &[(&str, PlaceLevel)],
    to_last_folder: bool,
) -> PathBuf {
    let deepest = folders.iter().map(|(_, level)| *level).max();
    let parts = [
        date.year().to_string(),
        date.month().to_string(),
        date.day().to_string(),
    ];
    let mut dir = PathBuf::new();
    for (level, part) in PlaceLevel::ALL.into_iter().zip(parts) {
        dir.push(part);
        for (name, folder_level) in folders {
            if *folder_level == level {
                dir.push(name);
            }
        }
        if to_last_folder && Some(level) == deepest {
            break;
        }
    }
    dir
}
