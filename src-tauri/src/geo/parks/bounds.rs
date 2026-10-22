//! Latitude and longitude boxes that let a lookup skip far parks and edges.

use crate::geo::grid::KM_PER_DEGREE;
use crate::geo::Coordinates;

#[derive(Debug, Clone, Copy)]
pub(super) struct Bounds {
    pub(super) south: f32,
    pub(super) west: f32,
    pub(super) north: f32,
    pub(super) east: f32,
}

impl Bounds {
    /// The box around `(latitude, longitude)` points, `None` for no points.
    pub(super) fn of<'a>(points: impl IntoIterator<Item = &'a (f32, f32)>) -> Option<Self> {
        let mut points = points.into_iter();
        let &(latitude, longitude) = points.next()?;
        let start = Self {
            south: latitude,
            west: longitude,
            north: latitude,
            east: longitude,
        };
        Some(points.fold(start, |bounds, &(latitude, longitude)| Self {
            south: bounds.south.min(latitude),
            west: bounds.west.min(longitude),
            north: bounds.north.max(latitude),
            east: bounds.east.max(longitude),
        }))
    }

    pub(super) fn widened(self, degrees: f32) -> Self {
        Self {
            south: self.south - degrees,
            west: self.west - degrees,
            north: self.north + degrees,
            east: self.east + degrees,
        }
    }

    pub(super) fn covers(self, point: Coordinates) -> bool {
        let (latitude, longitude) = (point.latitude as f32, point.longitude as f32);
        (self.south..=self.north).contains(&latitude)
            && (self.west..=self.east).contains(&longitude)
    }

    /// Whether a ray going east from `point` can cross an edge inside the box.
    pub(super) fn meets_eastward_ray(self, point: Coordinates) -> bool {
        let (latitude, longitude) = (point.latitude as f32, point.longitude as f32);
        (self.south..=self.north).contains(&latitude) && longitude <= self.east
    }
}

/// Degrees that span at least `km` around `point` in both directions. Degrees
/// of longitude shrink toward the poles, so this uses the longitude ones.
pub(super) fn margin_degrees(point: Coordinates, km: f64) -> f32 {
    let cos = point.latitude.to_radians().cos().max(0.01);
    (km / (KM_PER_DEGREE * cos)) as f32
}
