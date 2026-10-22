//! The places within 50 km of a point, gathered once and shared by every lookup.

use super::distance::Coordinates;
use super::index::{Entry, GeoIndex};
use super::kind::PlaceKind;
use super::radius::{city_reach_km, CITY_RADIUS_KM};

pub(super) struct Nearby<'a> {
    pub(super) index: &'a GeoIndex,
    pub(super) point: Coordinates,
    entries: Vec<(&'a Entry, f64)>,
}

impl<'a> Nearby<'a> {
    pub(super) fn new(index: &'a GeoIndex, point: Coordinates) -> Self {
        let entries = index.near(point, CITY_RADIUS_KM).collect();
        Self {
            index,
            point,
            entries,
        }
    }

    /// Cities whose reach covers the point, with how deep inside it sits: 0 at
    /// the centre, 1 at the edge.
    pub(super) fn covering_cities(&self) -> impl Iterator<Item = (&'a Entry, f64)> + '_ {
        self.entries
            .iter()
            .filter(|(entry, _)| entry.kind == PlaceKind::City)
            .map(|&(entry, distance)| (entry, distance / city_reach_km(entry.population)))
            .filter(|(_, depth)| *depth <= 1.0)
    }

    /// The most nested accepted place whose radius covers the point wins, so a
    /// garden beats the park around it. On a tie, the closest relative to its
    /// radius wins.
    pub(super) fn matching_poi(&self, accepts: impl Fn(PlaceKind) -> bool) -> Option<&'a Entry> {
        self.entries
            .iter()
            .filter(|(entry, _)| accepts(entry.kind))
            .filter_map(|&(entry, distance)| {
                let radius = entry.kind.match_radius_km()?;
                (distance <= radius).then_some((
                    entry,
                    entry.kind.nesting_level(),
                    distance / radius,
                ))
            })
            .min_by(|a, b| a.1.cmp(&b.1).then(a.2.total_cmp(&b.2)))
            .map(|(entry, _, _)| entry)
    }

    /// The nearest city within 50 km.
    pub(super) fn nearest_city(&self) -> Option<&'a Entry> {
        self.entries
            .iter()
            .filter(|(entry, _)| entry.kind == PlaceKind::City)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|&(entry, _)| entry)
    }
}
