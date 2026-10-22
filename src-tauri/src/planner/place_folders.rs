use std::collections::HashMap;

use chrono::NaiveDateTime;

use super::{place_folder::PlaceFolder, DestinationLayout};
use crate::{
    filename::validated_folder_name,
    models::{MetadataResult, PlaceSuggestion, SuggestionKind},
    trips::{day_layout_for_span, CHOSEN_DATE_FORMAT},
};

/// Place folders by file id, from the suggestions the person accepted.
#[derive(Debug, Default)]
pub(crate) struct PlaceFolders(HashMap<String, PlaceFolder>);

impl PlaceFolders {
    /// Checks edited names and recomputes day layouts from the dates. Same
    /// folder mode ignores the suggestions.
    pub(crate) fn new(
        layout: &DestinationLayout,
        suggestions: &[PlaceSuggestion],
        metadata_results: &[MetadataResult],
    ) -> Result<Self, String> {
        let mut folders = Self::default();
        if matches!(layout, DestinationLayout::SameFolder) {
            return Ok(folders);
        }
        let dates: HashMap<&str, NaiveDateTime> = metadata_results
            .iter()
            .filter_map(|result| {
                Some((
                    result.file_id.as_str(),
                    parse(result.chosen_date.as_deref()?).ok()?,
                ))
            })
            .collect();
        for suggestion in suggestions {
            folders.add_suggestion(suggestion, &dates)?;
        }
        Ok(folders)
    }

    pub(super) fn get(&self, file_id: &str) -> Option<&PlaceFolder> {
        self.0.get(file_id)
    }

    fn add_suggestion(
        &mut self,
        suggestion: &PlaceSuggestion,
        dates: &HashMap<&str, NaiveDateTime>,
    ) -> Result<(), String> {
        let year = parse(&suggestion.start)?.format("%Y").to_string();
        let base = match suggestion.kind {
            SuggestionKind::Trip => vec![year, validated_folder_name(&suggestion.title)?],
            SuggestionKind::Outing if suggestion.places.len() == 1 => vec![year],
            SuggestionKind::Outing => {
                return Err(format!(
                    "Outing \"{}\" needs exactly one place.",
                    suggestion.title
                ))
            }
        };
        for group in &suggestion.places {
            let mut parts = base.clone();
            parts.push(validated_folder_name(&group.name)?);
            self.add_files(
                &group.file_ids,
                parts,
                (group.start.as_str(), group.end.as_str()),
                dates,
            )?;
        }
        let trip_span = (suggestion.start.as_str(), suggestion.end.as_str());
        self.add_files(&suggestion.root_file_ids, base, trip_span, dates)
    }

    /// Day folders follow the suggested span widened by the real file dates,
    /// so every batch of one suggestion gets the same layout.
    fn add_files(
        &mut self,
        file_ids: &[String],
        parts: Vec<String>,
        (start, end): (&str, &str),
        dates: &HashMap<&str, NaiveDateTime>,
    ) -> Result<(), String> {
        let file_dates = file_ids.iter().filter_map(|id| dates.get(id.as_str()));
        let (first, last) = file_dates
            .fold((parse(start)?, parse(end)?), |(first, last), &date| {
                (first.min(date), last.max(date))
            });
        let day_layout = day_layout_for_span(first.date(), last.date());
        for file_id in file_ids {
            let folder = PlaceFolder {
                parts: parts.clone(),
                day_layout,
            };
            if self.0.insert(file_id.clone(), folder).is_some() {
                return Err(format!("File {file_id} is in more than one place folder."));
            }
        }
        Ok(())
    }
}

fn parse(date: &str) -> Result<NaiveDateTime, String> {
    NaiveDateTime::parse_from_str(date, CHOSEN_DATE_FORMAT)
        .map_err(|error| format!("Invalid place suggestion date {date}: {error}"))
}
