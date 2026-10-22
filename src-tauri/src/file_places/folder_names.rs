//! Folder names for the places of one scan.

use std::collections::{HashMap, HashSet};

use super::{
    place_table::{PlaceId, PlaceTable},
    shots::Shot,
};
use crate::{filename::sanitize_for_windows_filename, models::FilePlace};

/// The matched place or city of each geotagged shot. A place outside home's country gets
/// the country folder, and every country does with no known home. Different places that
/// share a folder get their region appended, as in `Central Park (New York)`.
pub(super) fn folder_places(
    shots: &[Shot],
    places: &PlaceTable,
    home_country: Option<&str>,
) -> HashMap<String, FilePlace> {
    let shot_places: Vec<(&str, PlaceId)> = shots
        .iter()
        .filter_map(|shot| {
            let resolved = shot.location?;
            Some((shot.file_id, resolved.place.or(resolved.city)?))
        })
        .collect();
    let distinct: HashSet<PlaceId> = shot_places.iter().map(|&(_, place)| place).collect();
    let mut folders: HashMap<PlaceId, FilePlace> = distinct
        .iter()
        .map(|&id| (id, base_folder(places, id, home_country)))
        .collect();
    let mut sharing: HashMap<FilePlace, usize> = HashMap::new();
    for folder in folders.values() {
        *sharing.entry(folder.clone()).or_insert(0) += 1;
    }
    for (&id, folder) in &mut folders {
        if sharing[folder] > 1 {
            let place = places.get(id);
            let region = place.admin1.as_deref().unwrap_or(&place.country_name);
            folder.place = sanitize_for_windows_filename(&format!("{} ({region})", folder.place));
        }
    }
    shot_places
        .into_iter()
        .map(|(file_id, place)| (file_id.to_owned(), folders[&place].clone()))
        .collect()
}

fn base_folder(places: &PlaceTable, id: PlaceId, home_country: Option<&str>) -> FilePlace {
    let place = places.get(id);
    let abroad = home_country.is_none_or(|home| !home.eq_ignore_ascii_case(&place.country_name));
    FilePlace {
        place: sanitize_for_windows_filename(&place.name),
        country: abroad.then(|| sanitize_for_windows_filename(&place.country_name)),
    }
}
