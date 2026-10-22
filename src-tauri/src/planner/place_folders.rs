use std::{collections::HashMap, path::PathBuf};

use chrono::{Datelike, NaiveDate, NaiveDateTime};

use super::place_level::{dated_dir, DaySpan, PlaceLevel};
use crate::{
    file_places::CHOSEN_DATE_FORMAT,
    filename::{place_label, validated_folder_name},
    models::{FilePlace, HomeMatch, MetadataResult},
};

/// The days a country or place folder's files span within one year, and its date level.
#[derive(Debug)]
pub(super) struct FolderSpan {
    pub(super) days: DaySpan,
    pub(super) level: PlaceLevel,
}

pub(super) type CountryKey = (i32, String);
pub(super) type PlaceKey = (i32, Option<String>, String);

/// The place folder of each file, under `<year>` and the date folders the country and the
/// place span within that year. Spans count only the files of this scan. Files in home's
/// city have no place folders, though their file names still name the place.
#[derive(Debug, Default)]
pub(crate) struct PlaceFolders {
    pub(super) by_file: HashMap<String, (FilePlace, PlaceKey)>,
    /// `<place>, <country>` for the file name of each file with a place, home's city included.
    labels: HashMap<String, String>,
    pub(super) countries: HashMap<CountryKey, FolderSpan>,
    pub(super) places: HashMap<PlaceKey, FolderSpan>,
}

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
        let mut by_file = HashMap::new();
        let mut labels = HashMap::new();
        let mut countries: HashMap<CountryKey, DaySpan> = HashMap::new();
        let mut places: HashMap<PlaceKey, DaySpan> = HashMap::new();
        for (file_id, found) in file_places {
            let Some(&date) = dates.get(file_id.as_str()) else {
                continue;
            };
            let place = validated(found)?;
            if let Some(label) = place_label(&place.place, &place.country) {
                labels.insert(file_id.clone(), label);
            }
            if found.home == HomeMatch::City {
                continue;
            }
            let year = date.year();
            let country = place.country_folder().map(str::to_owned);
            if let Some(country) = &country {
                countries
                    .entry((year, country.clone()))
                    .or_default()
                    .add(date);
            }
            let key = (year, country, place.place.clone());
            places.entry(key.clone()).or_default().add(date);
            by_file.insert(file_id.clone(), (place, key));
        }
        Ok(Self {
            by_file,
            labels,
            countries: leveled(countries),
            places: leveled(places),
        })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.by_file.is_empty()
    }

    /// The place of `file_id`, as the plan records it.
    pub(super) fn place(&self, file_id: &str) -> Option<&FilePlace> {
        self.by_file.get(file_id).map(|(place, _)| place)
    }

    /// The place part of the file name of `file_id`.
    pub(super) fn label(&self, file_id: &str) -> Option<&str> {
        self.labels.get(file_id).map(String::as_str)
    }

    /// `<year>`, `<month>` and `<day>` with the country and place folders at their level.
    pub(super) fn relative_dir(&self, file_id: &str, date: &NaiveDateTime) -> Option<PathBuf> {
        Some(dated_dir(date.date(), &self.folders(file_id)?, false))
    }

    /// What decides a file's folder and name apart from its date, so a file is planned again
    /// only when it changes.
    pub(crate) fn key(&self, file_id: &str) -> String {
        format!("{:?}:{:?}", self.folders(file_id), self.label(file_id))
    }

    /// The country folder, when there is one, then the place folder, each with its level.
    fn folders(&self, file_id: &str) -> Option<Vec<(&str, PlaceLevel)>> {
        let (_, key) = self.by_file.get(file_id)?;
        Some(self.key_folders(key))
    }

    pub(super) fn key_folders<'a>(&'a self, key: &'a PlaceKey) -> Vec<(&'a str, PlaceLevel)> {
        let (year, country, place) = key;
        let mut folders = Vec::with_capacity(2);
        if let Some(country) = country {
            let span = &self.countries[&(*year, country.clone())];
            folders.push((country.as_str(), span.level));
        }
        folders.push((place.as_str(), self.places[key].level));
        folders
    }
}

fn leveled<K: std::hash::Hash + Eq>(spans: HashMap<K, DaySpan>) -> HashMap<K, FolderSpan> {
    spans
        .into_iter()
        .map(|(key, days)| {
            let level = days.level();
            (key, FolderSpan { days, level })
        })
        .collect()
}

fn validated(place: &FilePlace) -> Result<FilePlace, String> {
    Ok(FilePlace {
        place: validated_folder_name(&place.place)?,
        country: validated_folder_name(&place.country)?,
        home: place.home,
    })
}
