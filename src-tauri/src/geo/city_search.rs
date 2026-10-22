//! Finds cities by name, for picking home in settings.

use std::cmp::Reverse;
use std::sync::LazyLock;

use super::fold::fold;
use super::index::INDEX;
use super::kind::PlaceKind;
use super::place::Place;

/// Folded city names, biggest city first, with their entry position.
// Built on the first search, so neither startup nor reverse geocoding pays for it.
static CITY_NAMES: LazyLock<Vec<(String, usize)>> = LazyLock::new(|| {
    let index = &*INDEX;
    let entries = index.entries();
    let mut positions: Vec<usize> = (0..entries.len())
        .filter(|&position| entries[position].kind == PlaceKind::City)
        .collect();
    positions.sort_by_key(|&position| Reverse(entries[position].population));
    positions
        .into_iter()
        .map(|position| (fold(index.name(&entries[position])), position))
        .collect()
});

/// Up to `limit` cities whose name starts with `query`, then ones that contain
/// it, biggest first. Case and accents are ignored.
pub(crate) fn search_cities(query: &str, limit: usize) -> Vec<Place> {
    let query = fold(query.trim());
    if query.is_empty() {
        return Vec::new();
    }
    let names = &*CITY_NAMES;
    let starting = names.iter().filter(|(name, _)| name.starts_with(&query));
    let containing = names
        .iter()
        .filter(|(name, _)| !name.starts_with(&query) && name.contains(&query));
    let index = &*INDEX;
    starting
        .chain(containing)
        .take(limit)
        .map(|(_, position)| index.place(&index.entries()[*position]))
        .collect()
}
