use std::collections::HashMap;

use chrono::NaiveDateTime;

use super::{
    place_dates::parse_date, place_folder::PlaceFolder, suggestion_folders::suggestion_folders,
};
use crate::models::{MetadataResult, PlaceSuggestion};

/// Place folders by file id, from the suggestions the person accepted.
#[derive(Debug, Default)]
pub(crate) struct PlaceFolders {
    by_file: HashMap<String, PlaceFolder>,
    /// Folders of suggestions that continue a placed trip or outing.
    continued: Vec<PlaceFolder>,
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
                if found.continued {
                    folders.continued.push(found.folder);
                }
            }
        }
        Ok(folders)
    }

    pub(super) fn get(&self, file_id: &str) -> Option<&PlaceFolder> {
        self.by_file.get(file_id)
    }

    pub(super) fn continued(&self) -> &[PlaceFolder] {
        &self.continued
    }
}
