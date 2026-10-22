//! Park outlines from OpenStreetMap, through Overture Maps (ODbL).
//!
//! `scripts/geodata/build_parks.py` builds `assets/parks.tsv.gz`. The outlines
//! load on the first lookup, apart from the GeoNames table.

mod asset;
mod bounds;
mod outline;
mod polyline;

use std::sync::LazyLock;
use std::time::Instant;

use super::distance::Coordinates;
use super::grid::{cell_of, CELL_COUNT, LON_CELLS};
use super::kind::PlaceKind;
use bounds::margin_degrees;
use outline::Outline;

static ASSET: &[u8] = include_bytes!("../../../assets/parks.tsv.gz");

pub(super) static PARKS: LazyLock<Parks> = LazyLock::new(|| {
    let started = Instant::now();
    let parks = Parks::new(asset::parse(ASSET));
    log::info!(
        "Loaded {} park outlines in {:?}",
        parks.parks.len(),
        started.elapsed()
    );
    parks
});

/// Parks join the cells this close to their bounds too, so `near` finds a park
/// across a cell edge by looking in one cell.
const NEAR_MARGIN_DEGREES: f32 = 0.1;
/// Below `NEAR_MARGIN_DEGREES` at latitudes people photograph.
const MAX_NEAR_KM: f64 = 5.0;

pub(super) struct Park {
    pub(super) name: String,
    pub(super) kind: PlaceKind,
    pub(super) area_km2: f32,
    /// A point inside the park, used to find its country.
    pub(super) label: Coordinates,
    outline: Outline,
}

pub(super) struct Parks {
    parks: Vec<Park>,
    /// The parks whose bounds touch each one-degree cell.
    cells: Vec<Vec<u32>>,
}

impl Parks {
    fn new(parks: Vec<Park>) -> Self {
        let mut cells = vec![Vec::new(); CELL_COUNT];
        let corner = |latitude: f32, longitude: f32| {
            let longitude = longitude.clamp(-180.0, 179.99);
            cell_of(Coordinates::new(latitude.into(), longitude.into()))
        };
        for (id, park) in parks.iter().enumerate() {
            let bounds = park.outline.bounds.widened(NEAR_MARGIN_DEGREES);
            let first = corner(bounds.south, bounds.west);
            let last = corner(bounds.north, bounds.east);
            for row in first / LON_CELLS..=last / LON_CELLS {
                for column in first % LON_CELLS..=last % LON_CELLS {
                    cells[row * LON_CELLS + column].push(id as u32);
                }
            }
        }
        Self { parks, cells }
    }

    /// The smallest park that contains `point`.
    pub(super) fn containing(&self, point: Coordinates) -> Option<&Park> {
        self.in_cell(point)
            .filter(|park| park.outline.contains(point))
            .min_by(|a, b| a.area_km2.total_cmp(&b.area_km2))
    }

    /// Parks within `radius_km` of `point`, up to 5 km, with their distance.
    pub(super) fn near(
        &self,
        point: Coordinates,
        radius_km: f64,
    ) -> impl Iterator<Item = (&Park, f64)> + '_ {
        let radius_km = radius_km.min(MAX_NEAR_KM);
        let margin = margin_degrees(point, radius_km);
        self.in_cell(point)
            .filter(move |park| park.outline.bounds.widened(margin).covers(point))
            .map(move |park| {
                let distance = if park.outline.contains(point) {
                    0.0
                } else {
                    park.outline.distance_km(point, radius_km)
                };
                (park, distance)
            })
            .filter(move |(_, distance)| *distance <= radius_km)
    }

    fn in_cell(&self, point: Coordinates) -> impl Iterator<Item = &Park> + '_ {
        self.cells[cell_of(point)]
            .iter()
            .map(|&id| &self.parks[id as usize])
    }
}
