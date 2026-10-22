//! Suggests place folders: trips far from home and outings to notable places.
//!
//! Pure grouping over file dates and GPS points. It makes no filesystem calls.
//! Geocoding loads the place table on first use, so call it off the UI thread.

mod country_split;
mod day_layout;
mod draft;
mod home;
mod outings;
mod place_names;
mod place_scan;
mod place_table;
mod shots;
mod suggestion_build;
mod suggestion_id;
mod thresholds;
mod trip_places;
mod trip_runs;
mod trip_title;
mod untagged;

#[cfg(test)]
mod tests;

pub(crate) use day_layout::day_layout_for_span;
pub(crate) use home::Home;
pub(crate) use place_scan::PlaceScan;
pub(crate) use shots::CHOSEN_DATE_FORMAT;
pub(crate) use thresholds::TRIP_MAX_GAP;
