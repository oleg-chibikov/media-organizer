use serde_json::{Map, Value};

use crate::models::GpsPoint;

/// Reads the capture location from one ExifTool JSON row.
///
/// Photos keep it in `GPSLatitude` and `GPSLongitude`. Phone videos keep it in
/// `GPSCoordinates`, as "lat lon alt" numbers or as an ISO 6709 string.
pub(super) fn read_gps(object: &Map<String, Value>) -> Option<GpsPoint> {
    let latitude = object.get("GPSLatitude").and_then(number);
    let longitude = object.get("GPSLongitude").and_then(number);
    if let (Some(latitude), Some(longitude)) = (latitude, longitude) {
        return gps_point(latitude, longitude);
    }
    let coordinates = object.get("GPSCoordinates")?.as_str()?.trim();
    parse_space_separated(coordinates).or_else(|| parse_iso6709(coordinates))
}

/// Drops values outside the globe and the (0, 0) that cameras write without a fix.
fn gps_point(latitude: f64, longitude: f64) -> Option<GpsPoint> {
    let in_range = (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude);
    let is_null_island = latitude == 0.0 && longitude == 0.0;
    (in_range && !is_null_island).then_some(GpsPoint {
        latitude,
        longitude,
    })
}

fn number(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str()?.trim().parse().ok())
}

fn parse_space_separated(text: &str) -> Option<GpsPoint> {
    let mut parts = text.split_whitespace().map(str::parse::<f64>);
    let latitude = parts.next()?.ok()?;
    let longitude = parts.next()?.ok()?;
    gps_point(latitude, longitude)
}

/// Reads the decimal degrees form, such as "+37.7749-122.4194+010.000/".
fn parse_iso6709(text: &str) -> Option<GpsPoint> {
    let body = text.trim().trim_end_matches('/');
    let starts: Vec<usize> = body.match_indices(['+', '-']).map(|(at, _)| at).collect();
    if starts.first() != Some(&0) || starts.len() < 2 {
        return None;
    }
    let end_of_longitude = starts.get(2).copied().unwrap_or(body.len());
    let latitude = body[starts[0]..starts[1]].parse().ok()?;
    let longitude = body[starts[1]..end_of_longitude].parse().ok()?;
    gps_point(latitude, longitude)
}
