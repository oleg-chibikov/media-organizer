//! Why a scan gave no place folders, in words the person can act on.

use super::{home::Home, shots::Shot, thresholds::TRIP_MIN_DISTANCE_KM};
use crate::{geo::distance_km, models::DetectionStatus};

/// The first reason that applies: GPS, then home, then distance, then file counts.
pub(super) fn detection_status(
    shots: &[Shot],
    home: Option<&Home>,
    found_any: bool,
) -> DetectionStatus {
    if found_any {
        return DetectionStatus::Found;
    }
    let mut points = shots
        .iter()
        .filter_map(|shot| shot.location.map(|location| location.point))
        .peekable();
    if points.peek().is_none() {
        return DetectionStatus::NoGps;
    }
    let Some(home) = home else {
        return DetectionStatus::HomeUnknown;
    };
    if points.all(|point| distance_km(point, home.point) <= TRIP_MIN_DISTANCE_KM) {
        return DetectionStatus::AllNearHome {
            home: home.name.clone(),
        };
    }
    DetectionStatus::TooFewGeotagged
}
