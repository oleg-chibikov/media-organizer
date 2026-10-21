use serde_json::{json, Value};

use crate::{
    metadata::{exiftool_json::parse_exiftool_json, gps::read_gps},
    models::GpsPoint,
};

fn gps_of(row: Value) -> Option<GpsPoint> {
    read_gps(row.as_object().expect("row is an object"))
}

fn point(latitude: f64, longitude: f64) -> Option<GpsPoint> {
    Some(GpsPoint {
        latitude,
        longitude,
    })
}

#[test]
fn reads_signed_decimal_degrees() {
    let north_east = json!({"GPSLatitude": 48.8584, "GPSLongitude": 2.2945});
    let south_west = json!({"GPSLatitude": -33.8568, "GPSLongitude": "-70.6483"});

    assert_eq!(gps_of(north_east), point(48.8584, 2.2945));
    assert_eq!(gps_of(south_west), point(-33.8568, -70.6483));
}

#[test]
fn needs_both_coordinates() {
    assert_eq!(gps_of(json!({"GPSLatitude": 48.8584})), None);
    assert_eq!(gps_of(json!({"GPSLongitude": 2.2945})), None);
}

#[test]
fn drops_null_island_and_out_of_range_values() {
    assert_eq!(gps_of(json!({"GPSLatitude": 0, "GPSLongitude": 0})), None);
    assert_eq!(gps_of(json!({"GPSLatitude": 91, "GPSLongitude": 10})), None);
    assert_eq!(
        gps_of(json!({"GPSLatitude": 10, "GPSLongitude": -181})),
        None
    );
    assert_eq!(
        gps_of(json!({"GPSCoordinates": "+3746.49-12225.16/"})),
        None
    );
}

#[test]
fn reads_video_coordinates() {
    let numbers = json!({"GPSCoordinates": "37.7749 -122.4194 10"});
    let iso6709 = json!({"GPSCoordinates": "+37.7749-122.4194+010.000/"});
    let without_altitude = json!({"GPSCoordinates": "-33.8568+151.2153/"});

    assert_eq!(gps_of(numbers), point(37.7749, -122.4194));
    assert_eq!(gps_of(iso6709), point(37.7749, -122.4194));
    assert_eq!(gps_of(without_altitude), point(-33.8568, 151.2153));
}

#[test]
fn prefers_photo_tags_over_video_coordinates() {
    let row = json!({
        "GPSLatitude": 1.5,
        "GPSLongitude": 2.5,
        "GPSCoordinates": "3.5 4.5 0"
    });

    assert_eq!(gps_of(row), point(1.5, 2.5));
}

#[test]
fn ignores_text_that_is_not_coordinates() {
    assert_eq!(gps_of(json!({"GPSCoordinates": "Paris"})), None);
    assert_eq!(gps_of(json!({"GPSCoordinates": ""})), None);
}

#[test]
fn json_row_with_gps_fills_the_raw_metadata() {
    let json = r#"[{
        "SourceFile": "/todo/yosemite.jpg",
        "DateTimeOriginal": "2025:10:28 09:00:00",
        "GPSLatitude": 37.7456,
        "GPSLongitude": -119.5936
    }]"#;

    let parsed = parse_exiftool_json(json).expect("json should parse");

    assert_eq!(parsed.gps, point(37.7456, -119.5936));
}
