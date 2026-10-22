//! Geocodes GPS points once and gives each place a small id.

use std::collections::HashMap;

use super::thresholds::GEOCODE_CACHE_STEP_DEGREES;
use crate::geo::{self, Coordinates, Place};

pub(super) type PlaceId = usize;

/// The resolved place of a point and the city it sits in.
#[derive(Debug, Clone, Copy)]
pub(super) struct Resolved {
    pub(super) place: Option<PlaceId>,
    pub(super) city: Option<PlaceId>,
}

#[derive(Default)]
pub(super) struct PlaceTable {
    places: Vec<Place>,
    ids: HashMap<(String, u64, u64), PlaceId>,
    // Burst photos share a point, so nearby points reuse one lookup.
    cache: HashMap<(i64, i64), Resolved>,
}

impl PlaceTable {
    pub(super) fn resolve(&mut self, point: Coordinates) -> Resolved {
        let key = (cache_step(point.latitude), cache_step(point.longitude));
        if let Some(resolved) = self.cache.get(&key) {
            return *resolved;
        }
        let resolved = Resolved {
            place: geo::nearest_place(point).map(|place| self.intern(place)),
            city: geo::nearest_city(point).map(|place| self.intern(place)),
        };
        self.cache.insert(key, resolved);
        resolved
    }

    pub(super) fn get(&self, id: PlaceId) -> &Place {
        &self.places[id]
    }

    pub(super) fn intern(&mut self, place: Place) -> PlaceId {
        let key = (
            place.name.clone(),
            place.latitude.to_bits(),
            place.longitude.to_bits(),
        );
        *self.ids.entry(key).or_insert_with(|| {
            self.places.push(place);
            self.places.len() - 1
        })
    }
}

fn cache_step(degrees: f64) -> i64 {
    (degrees / GEOCODE_CACHE_STEP_DEGREES).round() as i64
}
