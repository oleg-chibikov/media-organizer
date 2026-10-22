//! Folder names for the places of one scan.

use std::collections::{HashMap, HashSet};

use super::{
    home::Home,
    place_table::{PlaceId, PlaceTable},
    shots::Shot,
};
use crate::{
    filename::sanitize_for_windows_filename,
    geo::Place,
    models::{FilePlace, HomeMatch},
};

/// The matched place or city of each geotagged shot, with its country and how much of it is
/// home. Different places that share a name get their region appended, as in
/// `Central Park (New York)`.
pub(super) fn folder_places(
    shots: &[Shot],
    places: &PlaceTable,
    home: Option<&Home>,
) -> HashMap<String, FilePlace> {
    let shot_places: Vec<(&str, PlaceId, Option<PlaceId>)> = shots
        .iter()
        .filter_map(|shot| {
            let resolved = shot.location?;
            Some((
                shot.file_id,
                resolved.place.or(resolved.city)?,
                resolved.city,
            ))
        })
        .collect();
    let distinct: HashSet<PlaceId> = shot_places.iter().map(|&(_, place, _)| place).collect();
    let mut folders: HashMap<PlaceId, FilePlace> = distinct
        .iter()
        .map(|&id| (id, base_folder(places.get(id), home)))
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
        .map(|(file_id, place, city)| {
            let mut folder = folders[&place].clone();
            if city.is_some_and(|city| is_home_city(places.get(city), home)) {
                folder.home = HomeMatch::City;
            }
            (file_id.to_owned(), folder)
        })
        .collect()
}

fn base_folder(place: &Place, home: Option<&Home>) -> FilePlace {
    let in_home_country =
        home.is_some_and(|home| home.country.eq_ignore_ascii_case(&place.country_name));
    FilePlace {
        place: sanitize_for_windows_filename(&place.name),
        country: sanitize_for_windows_filename(&place.country_name),
        home: if in_home_country {
            HomeMatch::Country
        } else {
            HomeMatch::Away
        },
    }
}

fn is_home_city(city: &Place, home: Option<&Home>) -> bool {
    home.is_some_and(|home| {
        home.city.eq_ignore_ascii_case(&city.name)
            && home.country.eq_ignore_ascii_case(&city.country_name)
    })
}
