//! Short place labels for the file table, from the lookups grouping already made.

use std::collections::HashMap;

use super::{place_table::PlaceTable, shots::Shot};

/// The matched place or city of each geotagged shot. A country other than home's
/// is added, as in `Louvre, France`.
pub(super) fn file_places(
    shots: &[Shot],
    places: &PlaceTable,
    home_country: Option<&str>,
) -> HashMap<String, String> {
    shots
        .iter()
        .filter_map(|shot| {
            let resolved = shot.location?.resolved;
            let place = places.get(resolved.place.or(resolved.city)?);
            let abroad =
                home_country.is_some_and(|home| !home.eq_ignore_ascii_case(&place.country_name));
            let label = if abroad {
                format!("{}, {}", place.name, place.country_name)
            } else {
                place.name.clone()
            };
            Some((shot.file_id.to_owned(), label))
        })
        .collect()
}
