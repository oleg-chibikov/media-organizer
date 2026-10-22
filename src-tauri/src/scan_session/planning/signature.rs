use std::collections::HashMap;

use crate::models::{MetadataResult, PlaceSuggestion, SuggestionKind};

/// The place folder of each accepted file, as a key. A file absent from the map goes to
/// date folders.
pub(super) fn place_keys(suggestions: &[PlaceSuggestion]) -> HashMap<String, String> {
    let mut keys = HashMap::new();
    for suggestion in suggestions {
        let base = match suggestion.kind {
            SuggestionKind::Trip => suggestion.title.clone(),
            SuggestionKind::Outing => String::new(),
        };
        for place in &suggestion.places {
            let key = format!("{base}/{}", place.name);
            for file_id in &place.file_ids {
                keys.insert(file_id.clone(), key.clone());
            }
        }
        for file_id in &suggestion.root_file_ids {
            keys.insert(file_id.clone(), base.clone());
        }
    }
    keys
}

/// A file is planned again only when its date or its place folder changes its destination.
pub(super) fn plan_signature(metadata: &MetadataResult, place_key: Option<&String>) -> String {
    format!(
        "{:?}:{}:{}",
        metadata.metadata_status,
        metadata.chosen_date.as_deref().unwrap_or(""),
        place_key.map_or("", String::as_str)
    )
}
