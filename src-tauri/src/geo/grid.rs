//! A one-degree grid over the globe. Entries sorted by cell let a lookup
//! scan only the cells a search circle touches.

use std::ops::Range;

use super::distance::Coordinates;

const LAT_CELLS: usize = 180;
pub(super) const LON_CELLS: usize = 360;
pub(super) const CELL_COUNT: usize = LAT_CELLS * LON_CELLS;
pub(super) const KM_PER_DEGREE: f64 = 111.2;
// Above this latitude a search circle spans every longitude.
const POLAR_LATITUDE: f64 = 89.0;

pub(super) fn cell_of(point: Coordinates) -> usize {
    let row = ((point.latitude + 90.0).floor() as isize).clamp(0, LAT_CELLS as isize - 1);
    let column = ((point.longitude + 180.0).floor() as isize).rem_euclid(LON_CELLS as isize);
    row as usize * LON_CELLS + column as usize
}

pub(super) struct Grid {
    /// Entry range of cell `c` is `starts[c]..starts[c + 1]`.
    starts: Vec<u32>,
}

impl Grid {
    /// Takes the cell of every entry, in the sorted order of the entries.
    pub(super) fn from_sorted_cells(cells: impl Iterator<Item = usize>) -> Self {
        let mut starts = vec![0u32; CELL_COUNT + 1];
        for cell in cells {
            starts[cell + 1] += 1;
        }
        for cell in 1..starts.len() {
            starts[cell] += starts[cell - 1];
        }
        Self { starts }
    }

    /// Entry ranges of the cells within `radius_km` of `point`.
    pub(super) fn ranges_near(
        &self,
        point: Coordinates,
        radius_km: f64,
    ) -> impl Iterator<Item = Range<usize>> + '_ {
        let span = radius_km / KM_PER_DEGREE;
        let south = (point.latitude - span).max(-90.0);
        let north = (point.latitude + span).min(90.0);
        let widest = south.abs().max(north.abs());
        let lon_span = span / widest.to_radians().cos();
        let columns = if widest >= POLAR_LATITUDE || lon_span >= 179.0 {
            -180.0..=179.0
        } else {
            (point.longitude - lon_span)..=(point.longitude + lon_span)
        };
        let rows = row_of(south)..=row_of(north);
        let columns = column_index(*columns.start())..=column_index(*columns.end());
        rows.flat_map(move |row| {
            columns.clone().map(move |column| {
                let cell = row * LON_CELLS + column.rem_euclid(LON_CELLS as isize) as usize;
                self.starts[cell] as usize..self.starts[cell + 1] as usize
            })
        })
    }
}

fn row_of(latitude: f64) -> usize {
    ((latitude + 90.0).floor() as usize).min(LAT_CELLS - 1)
}

fn column_index(longitude: f64) -> isize {
    (longitude + 180.0).floor() as isize
}
