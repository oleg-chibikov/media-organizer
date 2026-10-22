//! The saved home: used for place suggestions, learned from large scans and
//! picked by name in settings.

mod detect;
mod search;

#[cfg(test)]
mod tests;

pub(crate) use detect::detect_place_suggestions;
pub(crate) use search::search_home_cities;
