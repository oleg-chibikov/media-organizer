//! A park's rings, and whether a point sits inside or near them.

use super::bounds::{margin_degrees, Bounds};
use crate::geo::grid::KM_PER_DEGREE;
use crate::geo::Coordinates;

// Each run of edges keeps its own box, so a test skips most of a long ring.
const CHUNK_EDGES: usize = 64;

/// Outer rings and holes together.
pub(super) struct Outline {
    rings: Vec<Ring>,
    pub(super) bounds: Bounds,
}

struct Ring {
    /// `(latitude, longitude)` points. The last joins back to the first.
    points: Vec<(f32, f32)>,
    /// The box of each run of `CHUNK_EDGES` edges.
    chunks: Vec<Bounds>,
}

type Edge = ((f64, f64), (f64, f64));

impl Outline {
    pub(super) fn new(rings: Vec<Vec<(f32, f32)>>) -> Option<Self> {
        let bounds = Bounds::of(rings.iter().flatten())?;
        let rings = rings.into_iter().map(Ring::new).collect();
        Some(Self { rings, bounds })
    }

    /// An even-odd ray test. A point in a hole crosses an even number of edges.
    pub(super) fn contains(&self, point: Coordinates) -> bool {
        if !self.bounds.covers(point) {
            return false;
        }
        let (y, x) = (point.latitude, point.longitude);
        let crossings = self
            .edges_where(|chunk| chunk.meets_eastward_ray(point))
            .filter(|(a, b)| {
                (a.0 > y) != (b.0 > y) && x < (b.1 - a.1) * (y - a.0) / (b.0 - a.0) + a.1
            })
            .count();
        crossings % 2 == 1
    }

    /// Distance to the nearest edge on a flat map, close enough over a few km.
    /// Edges farther than `radius_km` are skipped, so a farther result is infinite.
    pub(super) fn distance_km(&self, point: Coordinates, radius_km: f64) -> f64 {
        let margin = margin_degrees(point, radius_km);
        let x_scale = point.latitude.to_radians().cos();
        let flat = |(latitude, longitude): (f64, f64)| {
            (
                (latitude - point.latitude) * KM_PER_DEGREE,
                (longitude - point.longitude) * KM_PER_DEGREE * x_scale,
            )
        };
        self.edges_where(|chunk| chunk.widened(margin).covers(point))
            .map(|(a, b)| distance_to_segment(flat(a), flat(b)))
            .fold(f64::INFINITY, f64::min)
    }

    fn edges_where<'a>(
        &'a self,
        wanted: impl Fn(Bounds) -> bool + Copy + 'a,
    ) -> impl Iterator<Item = Edge> + 'a {
        self.rings.iter().flat_map(move |ring| {
            ring.chunks
                .iter()
                .enumerate()
                .filter(move |(_, chunk)| wanted(**chunk))
                .flat_map(move |(index, _)| ring.chunk_edges(index))
        })
    }
}

impl Ring {
    fn new(points: Vec<(f32, f32)>) -> Self {
        let chunks = (0..points.len().div_ceil(CHUNK_EDGES))
            .map(|index| {
                let ends = Self::chunk_range(index, points.len())
                    .flat_map(|edge| [&points[edge], &points[(edge + 1) % points.len()]]);
                Bounds::of(ends).expect("a chunk has edges")
            })
            .collect();
        Self { points, chunks }
    }

    fn chunk_edges(&self, index: usize) -> impl Iterator<Item = Edge> + '_ {
        let widen = |(latitude, longitude): (f32, f32)| (f64::from(latitude), f64::from(longitude));
        let count = self.points.len();
        Self::chunk_range(index, count).map(move |edge| {
            (
                widen(self.points[edge]),
                widen(self.points[(edge + 1) % count]),
            )
        })
    }

    fn chunk_range(index: usize, count: usize) -> std::ops::Range<usize> {
        index * CHUNK_EDGES..((index + 1) * CHUNK_EDGES).min(count)
    }
}

/// Distance from the origin to the segment from `a` to `b`.
fn distance_to_segment(a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length_squared = dx * dx + dy * dy;
    let t = if length_squared == 0.0 {
        0.0
    } else {
        (-(a.0 * dx + a.1 * dy) / length_squared).clamp(0.0, 1.0)
    };
    (a.0 + t * dx).hypot(a.1 + t * dy)
}
