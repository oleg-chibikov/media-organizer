use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::models::{GpsCoordinate, LocationBucket, MetadataResult, PlaceCacheEntry, PlaceStatus};

pub const LOCATION_BUCKET_PRECISION_VERSION: u32 = 1;
pub const PLACE_CACHE_VERSION: u32 = 1;

const CACHE_DIR_NAME: &str = "cache";
const PLACE_CACHE_FILE_NAME: &str = "place-cache.json";
const CITY_BUCKET_STEP_DEGREES: f64 = 0.1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlaceCacheFile {
    version: u32,
    entries: Vec<PlaceCacheEntry>,
}

pub fn place_cache_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(CACHE_DIR_NAME)
}

pub fn place_cache_path(app_data_dir: &Path) -> PathBuf {
    place_cache_dir(app_data_dir).join(PLACE_CACHE_FILE_NAME)
}

pub fn bucket_for_coordinate(coordinate: &GpsCoordinate) -> Option<LocationBucket> {
    if !is_valid_coordinate(coordinate) {
        return None;
    }

    let latitude_index = quantize_coordinate(coordinate.latitude);
    let longitude_index = quantize_coordinate(coordinate.longitude);
    let bucket_id =
        format!("v{LOCATION_BUCKET_PRECISION_VERSION}:{latitude_index}:{longitude_index}");

    Some(LocationBucket {
        bucket_id,
        representative_coordinate: GpsCoordinate {
            latitude: bucket_center_from_index(latitude_index, -90.0, 90.0),
            longitude: bucket_center_from_index(longitude_index, -180.0, 180.0),
        },
        precision: LOCATION_BUCKET_PRECISION_VERSION,
        media_count: 1,
    })
}

pub fn load_place_cache(app_data_dir: &Path) -> Result<HashMap<String, PlaceCacheEntry>, String> {
    let cache_path = place_cache_path(app_data_dir);
    if !cache_path.exists() {
        return Ok(HashMap::new());
    }

    let raw = fs::read_to_string(&cache_path).map_err(|error| {
        format!(
            "Failed to read place cache file at {}: {error}",
            cache_path.display()
        )
    })?;

    let cache_file: PlaceCacheFile = serde_json::from_str(&raw).map_err(|error| {
        format!(
            "Failed to parse place cache file at {}: {error}",
            cache_path.display()
        )
    })?;

    if cache_file.version != PLACE_CACHE_VERSION {
        return Ok(HashMap::new());
    }

    let mut entries = HashMap::new();
    for mut entry in cache_file.entries {
        if entry.version != PLACE_CACHE_VERSION || entry.bucket_id.trim().is_empty() {
            continue;
        }
        entry.version = PLACE_CACHE_VERSION;
        entries.insert(entry.bucket_id.clone(), entry);
    }

    Ok(entries)
}

pub fn save_place_cache(
    app_data_dir: &Path,
    entries: &HashMap<String, PlaceCacheEntry>,
) -> Result<(), String> {
    let cache_path = place_cache_path(app_data_dir);
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "Failed to create place cache directory at {}: {error}",
                parent.display()
            )
        })?;
    }

    let mut ordered_entries: Vec<PlaceCacheEntry> = entries.values().cloned().collect();
    ordered_entries.sort_by(|left, right| left.bucket_id.cmp(&right.bucket_id));
    for entry in &mut ordered_entries {
        entry.version = PLACE_CACHE_VERSION;
    }

    let cache_file = PlaceCacheFile {
        version: PLACE_CACHE_VERSION,
        entries: ordered_entries,
    };
    let serialized = serde_json::to_string_pretty(&cache_file)
        .map_err(|error| format!("Failed to serialize place cache JSON: {error}"))?;

    fs::write(&cache_path, serialized).map_err(|error| {
        format!(
            "Failed to write place cache file at {}: {error}",
            cache_path.display()
        )
    })
}

pub fn apply_cache_to_metadata_results(
    app_data_dir: &Path,
    results: &mut [MetadataResult],
) -> Result<(), String> {
    if results.is_empty() {
        return Ok(());
    }

    let mut cache = load_place_cache(app_data_dir)?;
    let mut cache_was_updated = false;
    let last_used_at = Utc::now().to_rfc3339();

    for result in results.iter_mut() {
        let Some(gps_coordinate) = result.gps_coordinate.as_ref() else {
            result.place_bucket_id = None;
            result.place_status = PlaceStatus::NotAvailable;
            result.place = None;
            result.place_error = None;
            continue;
        };

        let Some(bucket) = bucket_for_coordinate(gps_coordinate) else {
            result.place_bucket_id = None;
            result.place_status = PlaceStatus::NotAvailable;
            result.place = None;
            result.place_error = None;
            continue;
        };

        result.place_bucket_id = Some(bucket.bucket_id.clone());
        if let Some(entry) = cache.get_mut(&bucket.bucket_id) {
            result.place_status = PlaceStatus::Resolved;
            result.place = Some(entry.place.clone());
            result.place_error = None;

            if entry.last_used_at != last_used_at {
                entry.last_used_at = last_used_at.clone();
                cache_was_updated = true;
            }
        } else {
            result.place_status = PlaceStatus::Pending;
            result.place = None;
            result.place_error = None;
        }
    }

    if cache_was_updated {
        save_place_cache(app_data_dir, &cache)?;
    }

    Ok(())
}

fn is_valid_coordinate(coordinate: &GpsCoordinate) -> bool {
    coordinate.latitude.is_finite()
        && coordinate.longitude.is_finite()
        && (-90.0..=90.0).contains(&coordinate.latitude)
        && (-180.0..=180.0).contains(&coordinate.longitude)
}

fn quantize_coordinate(value: f64) -> i32 {
    (value / CITY_BUCKET_STEP_DEGREES).floor() as i32
}

fn bucket_center_from_index(index: i32, min: f64, max: f64) -> f64 {
    ((index as f64 + 0.5) * CITY_BUCKET_STEP_DEGREES).clamp(min, max)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs};

    use tempfile::tempdir;

    use super::{
        apply_cache_to_metadata_results, bucket_for_coordinate, load_place_cache, place_cache_path,
        save_place_cache, LOCATION_BUCKET_PRECISION_VERSION, PLACE_CACHE_VERSION,
    };
    use crate::models::{
        GpsCoordinate, MetadataResult, MetadataStatus, PlaceCacheEntry, PlaceStatus,
    };

    #[test]
    fn generates_stable_bucket_for_same_coordinate() {
        let coordinate = GpsCoordinate {
            latitude: -33.8688,
            longitude: 151.2093,
        };

        let first = bucket_for_coordinate(&coordinate).expect("bucket should exist");
        let second = bucket_for_coordinate(&coordinate).expect("bucket should exist");

        assert_eq!(first.bucket_id, second.bucket_id);
        assert_eq!(first.precision, LOCATION_BUCKET_PRECISION_VERSION);
        assert_eq!(
            first.representative_coordinate.latitude,
            second.representative_coordinate.latitude
        );
        assert_eq!(
            first.representative_coordinate.longitude,
            second.representative_coordinate.longitude
        );
    }

    #[test]
    fn groups_nearby_coordinates_into_same_bucket() {
        let first = GpsCoordinate {
            latitude: -33.8688,
            longitude: 151.2093,
        };
        let second = GpsCoordinate {
            latitude: -33.8612,
            longitude: 151.2041,
        };

        let first_bucket = bucket_for_coordinate(&first).expect("first bucket should exist");
        let second_bucket = bucket_for_coordinate(&second).expect("second bucket should exist");

        assert_eq!(first_bucket.bucket_id, second_bucket.bucket_id);
    }

    #[test]
    fn separates_distant_coordinates_into_different_buckets() {
        let sydney = GpsCoordinate {
            latitude: -33.8688,
            longitude: 151.2093,
        };
        let melbourne = GpsCoordinate {
            latitude: -37.8136,
            longitude: 144.9631,
        };

        let sydney_bucket = bucket_for_coordinate(&sydney).expect("bucket should exist");
        let melbourne_bucket = bucket_for_coordinate(&melbourne).expect("bucket should exist");

        assert_ne!(sydney_bucket.bucket_id, melbourne_bucket.bucket_id);
    }

    #[test]
    fn persists_versioned_place_cache() {
        let temp = tempdir().expect("temp dir");
        let mut entries = HashMap::new();
        entries.insert(
            "v1:-339:1512".to_string(),
            PlaceCacheEntry {
                version: 99,
                bucket_id: "v1:-339:1512".to_string(),
                place: "Sydney, NSW, Australia".to_string(),
                components: None,
                provider: "cache-test".to_string(),
                resolved_at: "2026-05-25T00:00:00Z".to_string(),
                last_used_at: "2026-05-25T00:00:00Z".to_string(),
            },
        );

        save_place_cache(temp.path(), &entries).expect("cache should save");
        let loaded = load_place_cache(temp.path()).expect("cache should load");
        let persisted = loaded
            .get("v1:-339:1512")
            .expect("entry should round-trip with same bucket id");

        assert_eq!(persisted.version, PLACE_CACHE_VERSION);

        let raw = fs::read_to_string(place_cache_path(temp.path())).expect("cache file");
        let json: serde_json::Value = serde_json::from_str(&raw).expect("json should parse");
        assert_eq!(
            json.get("version").and_then(serde_json::Value::as_u64),
            Some(PLACE_CACHE_VERSION as u64)
        );
    }

    #[test]
    fn reuses_cached_place_entry_before_online_lookup() {
        let temp = tempdir().expect("temp dir");
        let coordinate = GpsCoordinate {
            latitude: -33.8688,
            longitude: 151.2093,
        };
        let bucket = bucket_for_coordinate(&coordinate).expect("bucket should exist");

        let mut entries = HashMap::new();
        entries.insert(
            bucket.bucket_id.clone(),
            PlaceCacheEntry {
                version: PLACE_CACHE_VERSION,
                bucket_id: bucket.bucket_id.clone(),
                place: "Sydney, NSW, Australia".to_string(),
                components: None,
                provider: "nominatim".to_string(),
                resolved_at: "2026-05-25T00:00:00Z".to_string(),
                last_used_at: "2026-05-25T00:00:00Z".to_string(),
            },
        );
        save_place_cache(temp.path(), &entries).expect("cache should save");

        let mut results = vec![metadata_result_with_coordinate("file-1", coordinate)];
        apply_cache_to_metadata_results(temp.path(), &mut results).expect("cache should apply");

        let resolved = &results[0];
        assert_eq!(
            resolved.place_bucket_id.as_deref(),
            Some(bucket.bucket_id.as_str())
        );
        assert_eq!(resolved.place_status, PlaceStatus::Resolved);
        assert_eq!(resolved.place.as_deref(), Some("Sydney, NSW, Australia"));
        assert!(resolved.place_error.is_none());

        let loaded = load_place_cache(temp.path()).expect("cache should reload");
        let refreshed = loaded
            .get(&bucket.bucket_id)
            .expect("bucket should remain in cache");
        assert_ne!(refreshed.last_used_at, "2026-05-25T00:00:00Z");
    }

    fn metadata_result_with_coordinate(file_id: &str, coordinate: GpsCoordinate) -> MetadataResult {
        MetadataResult {
            file_id: file_id.to_string(),
            chosen_date: None,
            chosen_date_source: None,
            date_kind: None,
            raw_metadata_date: None,
            device_name: None,
            gps_coordinate: Some(coordinate),
            gps_source: Some("GPSLatitude/GPSLongitude".to_string()),
            place_bucket_id: None,
            place_status: PlaceStatus::NotAvailable,
            place: None,
            place_error: None,
            metadata_status: MetadataStatus::Ready,
            error: None,
        }
    }
}
