use std::{collections::BTreeSet, fs, path::PathBuf};

use chrono::{Datelike, NaiveDate};

use super::{
    dir_probe::DirProbe,
    place_folders::{PlaceFolders, PlaceKey},
    place_level::{dated_dir, PlaceLevel},
};

/// A folder of the destination with files of one place from other days, relative to the root.
#[derive(Debug)]
pub(crate) struct NearbyFolder {
    pub(super) key: PlaceKey,
    pub(super) dir: PathBuf,
    /// Each day of the folder and the folder that holds its files.
    pub(super) days: Vec<(NaiveDate, PathBuf)>,
}

impl PlaceFolders {
    /// Finds folders of each place in its months and the months next to them, and counts
    /// their days in the place's span. Dubai on March 4 and March 7 then share `2024/3/Dubai`.
    pub(crate) fn gather_nearby(&mut self, root: &std::path::Path) -> Vec<NearbyFolder> {
        let mut probe = DirProbe::new(root);
        let mut found = Vec::new();
        for (key, span) in &self.places {
            let (year, country, name) = key;
            for month in nearby_months(&span.days.months()) {
                for layout in layouts(country.as_deref(), name) {
                    for (dir, days) in folders_in_month(&mut probe, *year, month, &layout) {
                        found.push(NearbyFolder {
                            key: key.clone(),
                            dir,
                            days,
                        });
                    }
                }
            }
        }
        for folder in &found {
            let (year, country, _) = &folder.key;
            for (date, _) in &folder.days {
                if let Some(span) = self.places.get_mut(&folder.key) {
                    span.days.add(*date);
                }
                let country = country.as_ref().map(|country| (*year, country.clone()));
                if let Some(span) = country.and_then(|key| self.countries.get_mut(&key)) {
                    span.days.add(*date);
                }
            }
        }
        for span in self.countries.values_mut().chain(self.places.values_mut()) {
            span.level = span.days.level();
        }
        found
    }
}

fn nearby_months(months: &BTreeSet<u32>) -> BTreeSet<u32> {
    months
        .iter()
        .flat_map(|month| month.saturating_sub(1)..=month + 1)
        .filter(|month| (1..=12).contains(month))
        .collect()
}

/// Each way an earlier run could have laid out the place below its year: the place in a
/// month or a day folder, with its country at the same level or above.
fn layouts<'a>(country: Option<&'a str>, place: &'a str) -> Vec<Vec<(&'a str, PlaceLevel)>> {
    let mut layouts = Vec::new();
    for place_level in [PlaceLevel::Month, PlaceLevel::Day] {
        let Some(country) = country else {
            layouts.push(vec![(place, place_level)]);
            continue;
        };
        for country_level in PlaceLevel::ALL.into_iter().filter(|l| *l <= place_level) {
            layouts.push(vec![(country, country_level), (place, place_level)]);
        }
    }
    layouts
}

fn folders_in_month(
    probe: &mut DirProbe,
    year: i32,
    month: u32,
    layout: &[(&str, PlaceLevel)],
) -> Vec<(PathBuf, Vec<(NaiveDate, PathBuf)>)> {
    let days_of_month = (1..=31).filter_map(|day| NaiveDate::from_ymd_opt(year, month, day));
    match layout.last().map(|(_, level)| *level) {
        Some(PlaceLevel::Day) => days_of_month
            .filter_map(|date| {
                let dir = dated_dir(date, layout, true);
                probe.is_dir(&dir).then(|| (dir.clone(), vec![(date, dir)]))
            })
            .collect(),
        Some(PlaceLevel::Month) => {
            let Some(first) = NaiveDate::from_ymd_opt(year, month, 1) else {
                return Vec::new();
            };
            let dir = dated_dir(first, layout, true);
            if !probe.is_dir(&dir) {
                return Vec::new();
            }
            let days = day_folders(probe, &dir, first);
            if days.is_empty() {
                return Vec::new();
            }
            vec![(dir, days)]
        }
        _ => Vec::new(),
    }
}

/// The `<day>` folders inside a month's place folder.
fn day_folders(
    probe: &DirProbe,
    dir: &std::path::Path,
    first: NaiveDate,
) -> Vec<(NaiveDate, PathBuf)> {
    let Ok(entries) = fs::read_dir(probe.root().join(dir)) else {
        return Vec::new();
    };
    let mut days: Vec<_> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| {
            let name = entry.file_name();
            let date = first.with_day(name.to_str()?.parse().ok()?)?;
            Some((date, dir.join(name)))
        })
        .collect();
    days.sort();
    days
}
