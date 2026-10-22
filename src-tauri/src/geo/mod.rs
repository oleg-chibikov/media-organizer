//! Offline reverse geocoding over a bundled GeoNames extract (CC BY 4.0) and
//! OpenStreetMap park outlines (ODbL).
//!
//! `scripts/geodata/build_geodata.py` builds `assets/geodata.tsv.gz`, and
//! `build_parks.py` builds `assets/parks.tsv.gz`. Both load on the first
//! lookup. Call lookups from a background thread.

mod area;
mod asset;
mod city_search;
mod distance;
mod fold;
mod grid;
mod index;
mod kind;
mod lookup;
mod nearby;
mod parks;
mod place;
mod radius;

#[cfg(test)]
mod tests;

pub(crate) use city_search::search_cities;
#[cfg(test)]
pub(crate) use distance::distance_km;
pub(crate) use distance::Coordinates;
#[cfg(test)]
pub(crate) use kind::PlaceKind;
#[cfg(test)]
pub(crate) use lookup::country_of;
pub(crate) use lookup::{nearest_city, nearest_place, resolve, Resolution};
pub(crate) use place::Place;
