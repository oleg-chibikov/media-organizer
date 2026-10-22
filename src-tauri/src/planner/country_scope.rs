use std::{collections::HashMap, path::Path};

use super::{place_dates::day_layout_for, place_folder::PlaceFolder};
use crate::{
    models::{ContinuedPlacement, Placement},
    placements::PlacedSuggestion,
};

/// The placed folders of one country that a country wide suggestion takes in: those of
/// its month, or every folder of the year when it sits at year level.
#[derive(Debug, Clone)]
pub(super) struct CountryScope {
    pub(super) year: i32,
    pub(super) month_folder: Option<String>,
    pub(super) country: String,
    pub(super) continued: ContinuedPlacement,
}

impl CountryScope {
    pub(super) fn same_as(&self, other: &Self) -> bool {
        self.year == other.year
            && self.month_folder == other.month_folder
            && self.country.eq_ignore_ascii_case(&other.country)
    }

    pub(super) fn takes(&self, root: &Path, record: &PlacedSuggestion) -> bool {
        let at_level = self.month_folder.is_none() || record.month_folder == self.month_folder;
        record.is_in(root)
            && record.year == self.year
            && at_level
            && record
                .country
                .as_deref()
                .is_some_and(|country| country.eq_ignore_ascii_case(&self.country))
    }

    /// Where the files of `record`'s `place` go: the same trip or outing and place at the
    /// scope's level. The day layout is the one of the accepted folder at that path, or
    /// else follows the placed span of the place, or of the whole country for trip roots.
    pub(super) fn folder_for(
        &self,
        record: &PlacedSuggestion,
        place: Option<&str>,
        accepted: &[PlaceFolder],
    ) -> Result<PlaceFolder, String> {
        let placement = Placement {
            year: self.year,
            month_folder: self.month_folder.clone(),
            ..record.placement(place)
        };
        if let Some(found) = accepted
            .iter()
            .find(|folder| folder.same_folder(&placement))
        {
            return Ok(found.clone());
        }
        let continued = &self.continued;
        let span = match place {
            Some(name) => continued
                .places
                .iter()
                .find(|placed| placed.name.eq_ignore_ascii_case(name))
                .map_or_else(Vec::new, |placed| vec![&placed.start, &placed.end]),
            None => vec![&continued.start, &continued.end],
        };
        Ok(PlaceFolder {
            day_layout: day_layout_for(&[], &span, &HashMap::new())?,
            placement,
        })
    }
}
