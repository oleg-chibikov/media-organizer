use std::collections::HashMap;

use chrono::NaiveDateTime;

use super::{
    continued_spans::continued_placement, country_scope::CountryScope, place_dates::parse_date,
    place_folder::PlaceFolder, suggestion_folders::suggestion_folders,
};
use crate::models::{MetadataResult, PlaceSuggestion};

/// Place folders by file id, from the suggestions the person accepted.
#[derive(Debug, Default)]
pub(crate) struct PlaceFolders {
    by_file: HashMap<String, PlaceFolder>,
    accepted: Vec<PlaceFolder>,
    /// Folders of suggestions that continue one placed trip or outing.
    continued: Vec<PlaceFolder>,
    /// Countries whose placed folders country wide suggestions continue.
    country_scopes: Vec<CountryScope>,
}

impl PlaceFolders {
    /// Checks edited names and recomputes day layouts from the dates.
    pub(crate) fn new(
        suggestions: &[PlaceSuggestion],
        metadata_results: &[MetadataResult],
    ) -> Result<Self, String> {
        let mut folders = Self::default();
        let dates: HashMap<&str, NaiveDateTime> = metadata_results
            .iter()
            .filter_map(|result| {
                Some((
                    result.file_id.as_str(),
                    parse_date(result.chosen_date.as_deref()?).ok()?,
                ))
            })
            .collect();
        for suggestion in suggestions {
            let continued = continued_placement(suggestion);
            for found in suggestion_folders(suggestion, &dates)? {
                for file_id in found.file_ids {
                    if folders
                        .by_file
                        .insert(file_id.clone(), found.folder.clone())
                        .is_some()
                    {
                        return Err(format!("File {file_id} is in more than one place folder."));
                    }
                }
                if continued.is_some_and(|placed| !placed.country_wide) {
                    folders.continued.push(found.folder.clone());
                }
                folders.accepted.push(found.folder);
            }
            if let (Some(placed), Some(country)) = (continued, &suggestion.country) {
                let scope = CountryScope {
                    year: placed.year,
                    month_folder: suggestion.month_folder.clone(),
                    country: country.clone(),
                    continued: placed.clone(),
                };
                // A trip and an outing in one country share its placed folders.
                if placed.country_wide
                    && !folders
                        .country_scopes
                        .iter()
                        .any(|known| known.same_as(&scope))
                {
                    folders.country_scopes.push(scope);
                }
            }
        }
        Ok(folders)
    }

    pub(super) fn get(&self, file_id: &str) -> Option<&PlaceFolder> {
        self.by_file.get(file_id)
    }

    pub(super) fn accepted(&self) -> &[PlaceFolder] {
        &self.accepted
    }

    pub(super) fn continued(&self) -> &[PlaceFolder] {
        &self.continued
    }

    pub(super) fn country_scopes(&self) -> &[CountryScope] {
        &self.country_scopes
    }

    /// Whether planning moves placed files at all.
    pub(super) fn continues_any(&self) -> bool {
        !self.continued.is_empty() || !self.country_scopes.is_empty()
    }
}
