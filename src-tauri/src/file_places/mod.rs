//! Finds the place folder of each photo and video: the matched place or city, inside a
//! country folder when the place is outside home's country.
//!
//! Pure lookups over file dates and GPS points. It makes no filesystem calls.
//! Geocoding loads the place table on first use, so call it off the UI thread.

mod folder_names;
mod home;
mod place_scan;
mod place_table;
mod shots;
mod thresholds;
mod untagged;

#[cfg(test)]
mod tests;

pub(crate) use home::Home;
pub(crate) use place_scan::PlaceScan;
pub(crate) use shots::CHOSEN_DATE_FORMAT;
