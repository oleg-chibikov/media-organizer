//! Great-circle distance between two points on the Earth.

use crate::models::GpsPoint;

const EARTH_RADIUS_KM: f64 = 6371.0088;

/// A point in decimal degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Coordinates {
    pub(crate) latitude: f64,
    pub(crate) longitude: f64,
}

impl Coordinates {
    pub(crate) const fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }
}

/// Haversine distance in kilometres.
pub(crate) fn distance_km(a: Coordinates, b: Coordinates) -> f64 {
    let (lat_a, lat_b) = (a.latitude.to_radians(), b.latitude.to_radians());
    let half_dlat = (lat_b - lat_a) / 2.0;
    let half_dlon = (b.longitude - a.longitude).to_radians() / 2.0;
    let h = half_dlat.sin().powi(2) + lat_a.cos() * lat_b.cos() * half_dlon.sin().powi(2);
    2.0 * EARTH_RADIUS_KM * h.sqrt().min(1.0).asin()
}

impl From<GpsPoint> for Coordinates {
    fn from(point: GpsPoint) -> Self {
        Self::new(point.latitude, point.longitude)
    }
}
