//! The in-memory place table, loaded once on first use.

use std::sync::LazyLock;
use std::time::Instant;

use super::distance::{distance_km, Coordinates};
use super::grid::{cell_of, Grid};
use super::kind::PlaceKind;
use super::place::Place;

pub(super) const NO_ADMIN1: u32 = u32::MAX;

static ASSET: &[u8] = include_bytes!("../../assets/geodata.tsv.gz");

// Built on the first lookup, which runs on a background thread, so startup stays fast.
pub(super) static INDEX: LazyLock<GeoIndex> = LazyLock::new(|| {
    let started = Instant::now();
    let index = super::asset::parse(ASSET);
    log::info!(
        "Loaded {} places for offline geocoding in {:?}",
        index.len(),
        started.elapsed()
    );
    index
});

/// One place, kept small: names live in one shared string.
pub(super) struct Entry {
    pub(super) latitude: f32,
    pub(super) longitude: f32,
    pub(super) kind: PlaceKind,
    pub(super) country: u16,
    pub(super) admin1: u32,
    pub(super) population: u32,
    pub(super) name_start: u32,
    pub(super) name_len: u16,
}

impl Entry {
    pub(super) fn coordinates(&self) -> Coordinates {
        Coordinates::new(f64::from(self.latitude), f64::from(self.longitude))
    }
}

/// Places plus the names they point at by index.
#[derive(Default)]
pub(super) struct Tables {
    pub(super) entries: Vec<Entry>,
    pub(super) names: String,
    pub(super) countries: Vec<(String, String)>,
    pub(super) admin1: Vec<String>,
}

pub(super) struct GeoIndex {
    tables: Tables,
    grid: Grid,
}

impl GeoIndex {
    pub(super) fn new(mut tables: Tables) -> Self {
        tables
            .entries
            .sort_by_key(|entry| cell_of(entry.coordinates()));
        let cells = tables
            .entries
            .iter()
            .map(|entry| cell_of(entry.coordinates()));
        let grid = Grid::from_sorted_cells(cells);
        Self { tables, grid }
    }

    pub(super) fn len(&self) -> usize {
        self.tables.entries.len()
    }

    /// Entries within `radius_km` of `point`, with their distance.
    pub(super) fn near(
        &self,
        point: Coordinates,
        radius_km: f64,
    ) -> impl Iterator<Item = (&Entry, f64)> + '_ {
        self.grid
            .ranges_near(point, radius_km)
            .flat_map(move |range| &self.tables.entries[range])
            .map(move |entry| (entry, distance_km(point, entry.coordinates())))
            .filter(move |(_, distance)| *distance <= radius_km)
    }

    pub(super) fn place(&self, entry: &Entry) -> Place {
        let tables = &self.tables;
        let start = entry.name_start as usize;
        let (country_code, country_name) = &tables.countries[usize::from(entry.country)];
        Place {
            name: tables.names[start..start + usize::from(entry.name_len)].to_owned(),
            kind: entry.kind,
            admin1: tables.admin1.get(entry.admin1 as usize).cloned(),
            country_code: country_code.clone(),
            country_name: country_name.clone(),
            latitude: f64::from(entry.latitude),
            longitude: f64::from(entry.longitude),
            population: entry.population,
        }
    }
}
