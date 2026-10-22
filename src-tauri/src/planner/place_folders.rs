use std::{collections::HashMap, path::PathBuf};

use chrono::{Datelike, NaiveDate, NaiveDateTime};

use super::place_level::{DaySpan, PlaceLevel};
use crate::{
    file_places::CHOSEN_DATE_FORMAT,
    filename::validated_folder_name,
    models::{FilePlace, MetadataResult},
};

/// One folder of a file's place: its name and the date folder it sits in.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Leveled {
    name: String,
    level: PlaceLevel,
}

/// The place folder of each file, under `<year>` and the date folders the country and the
/// place span within that year. Spans count only the files of this scan.
#[derive(Debug, Default)]
pub(crate) struct PlaceFolders {
    by_file: HashMap<String, (FilePlace, Option<Leveled>, Leveled)>,
}

type CountryKey = (i32, Option<String>);
type PlaceKey = (i32, Option<String>, String);

impl PlaceFolders {
    /// Fails on a folder name that isn't one plain name.
    pub(crate) fn new(
        file_places: &HashMap<String, FilePlace>,
        metadata_results: &[MetadataResult],
    ) -> Result<Self, String> {
        let dates: HashMap<&str, NaiveDate> = metadata_results
            .iter()
            .filter_map(|result| {
                let date = result.chosen_date.as_deref()?;
                let taken = NaiveDateTime::parse_from_str(date, CHOSEN_DATE_FORMAT).ok()?;
                Some((result.file_id.as_str(), taken.date()))
            })
            .collect();
        let mut files = Vec::new();
        let mut countries: HashMap<CountryKey, DaySpan> = HashMap::new();
        let mut places: HashMap<PlaceKey, DaySpan> = HashMap::new();
        for (file_id, found) in file_places {
            let Some(&date) = dates.get(file_id.as_str()) else {
                continue;
            };
            let place = validated(found)?;
            let year = date.year();
            if let Some(country) = &place.country {
                countries
                    .entry((year, Some(country.clone())))
                    .or_default()
                    .add(date);
            }
            places
                .entry((year, place.country.clone(), place.place.clone()))
                .or_default()
                .add(date);
            files.push((file_id.clone(), year, place));
        }
        let by_file = files
            .into_iter()
            .map(|(file_id, year, place)| {
                let country = place.country.clone().map(|name| Leveled {
                    level: countries[&(year, Some(name.clone()))].level(),
                    name,
                });
                let leveled = Leveled {
                    level: places[&(year, place.country.clone(), place.place.clone())].level(),
                    name: place.place.clone(),
                };
                (file_id, (place, country, leveled))
            })
            .collect();
        Ok(Self { by_file })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.by_file.is_empty()
    }

    /// The place of `file_id`, as the plan records it.
    pub(super) fn place(&self, file_id: &str) -> Option<&FilePlace> {
        self.by_file.get(file_id).map(|(place, _, _)| place)
    }

    /// `<year>`, `<month>` and `<day>` with the country and place folders at their level.
    pub(super) fn relative_dir(&self, file_id: &str, date: &NaiveDateTime) -> Option<PathBuf> {
        let (_, country, place) = self.by_file.get(file_id)?;
        let parts = [
            (PlaceLevel::Year, date.year().to_string()),
            (PlaceLevel::Month, date.month().to_string()),
            (PlaceLevel::Day, date.day().to_string()),
        ];
        let mut dir = PathBuf::new();
        for (level, part) in parts {
            dir.push(part);
            for folder in country.iter().chain([place]) {
                if folder.level == level {
                    dir.push(&folder.name);
                }
            }
        }
        Some(dir)
    }

    /// What decides a file's folder apart from its date, so a file is planned again only
    /// when it changes.
    pub(crate) fn key(&self, file_id: &str) -> String {
        self.by_file
            .get(file_id)
            .map(|(_, country, place)| format!("{country:?}/{place:?}"))
            .unwrap_or_default()
    }
}

fn validated(place: &FilePlace) -> Result<FilePlace, String> {
    Ok(FilePlace {
        place: validated_folder_name(&place.place)?,
        country: place
            .country
            .as_deref()
            .map(validated_folder_name)
            .transpose()?,
    })
}
