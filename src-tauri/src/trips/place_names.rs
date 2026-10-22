//! Folder names for the places of one suggestion.

use std::collections::HashMap;

use super::place_table::{PlaceId, PlaceTable};
use crate::filename::sanitize_for_windows_filename;

/// Safe folder names. Different places that share a name get their region
/// appended, as in `Central Park (New York)`.
pub(super) fn place_folder_names(group_places: &[PlaceId], places: &PlaceTable) -> Vec<String> {
    let base_names: Vec<String> = group_places
        .iter()
        .map(|&place| sanitize_for_windows_filename(&places.get(place).name))
        .collect();
    let mut name_counts: HashMap<&str, usize> = HashMap::new();
    for name in &base_names {
        *name_counts.entry(name.as_str()).or_insert(0) += 1;
    }
    group_places
        .iter()
        .zip(&base_names)
        .map(|(&place, name)| {
            if name_counts[name.as_str()] == 1 {
                return name.clone();
            }
            let place = places.get(place);
            let region = place.admin1.as_deref().unwrap_or(&place.country_name);
            sanitize_for_windows_filename(&format!("{name} ({region})"))
        })
        .collect()
}
