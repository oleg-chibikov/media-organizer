//! Offline reverse geocoding over a bundled GeoNames extract (CC BY 4.0).
//!
//! `scripts/geodata/build_geodata.py` builds `assets/geodata.tsv.gz`. The
//! table loads on the first lookup. Call lookups from a background thread.

mod asset;
mod distance;
mod grid;
mod index;
mod kind;
mod lookup;
mod place;
mod radius;

#[cfg(test)]
mod tests;

pub(crate) use distance::{distance_km, Coordinates};
pub(crate) use kind::PlaceKind;
#[cfg(test)]
pub(crate) use lookup::country_of;
pub(crate) use lookup::{nearest_city, nearest_place};
pub(crate) use place::Place;
