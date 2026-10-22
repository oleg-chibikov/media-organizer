use std::path::{Path, PathBuf};

use super::{
    place_folders::PlaceFolders,
    place_level::{dated_dir, PlaceLevel},
};

impl PlaceFolders {
    /// Raises each country and place folder to a higher date level where `root` already
    /// has it. One day in Osaka then joins `2024/3/Osaka` instead of a new `2024/3/15/Osaka`.
    pub(crate) fn reuse_existing(&mut self, root: &Path) {
        let Self {
            countries, places, ..
        } = self;
        for ((year, name), span) in countries.iter_mut() {
            let Some(date) = span.days.first_date(*year) else {
                continue;
            };
            let found = existing_level(root, span.level, |level| {
                Some(dated_dir(date, &[(name, level)], true))
            });
            span.level = found.unwrap_or(span.level);
        }
        for ((year, country, name), span) in places.iter_mut() {
            let Some(date) = span.days.first_date(*year) else {
                continue;
            };
            let country = country
                .as_ref()
                .map(|country| (country.as_str(), countries[&(*year, country.clone())].level));
            let found = existing_level(root, span.level, |level| {
                // A place folder sits at or below its country folder.
                if country.is_some_and(|(_, country_level)| level < country_level) {
                    return None;
                }
                let folders: Vec<_> = country
                    .into_iter()
                    .chain([(name.as_str(), level)])
                    .collect();
                Some(dated_dir(date, &folders, true))
            });
            span.level = found.unwrap_or(span.level);
        }
    }
}

/// The highest level above `current` whose folder is already under `root`.
fn existing_level(
    root: &Path,
    current: PlaceLevel,
    folder_at: impl Fn(PlaceLevel) -> Option<PathBuf>,
) -> Option<PlaceLevel> {
    PlaceLevel::ALL
        .into_iter()
        .filter(|level| *level < current)
        .find(|level| folder_at(*level).is_some_and(|dir| root.join(dir).is_dir()))
}
