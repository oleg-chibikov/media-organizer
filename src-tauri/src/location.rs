use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::models::{
    GpsCoordinate, LocationBucket, MetadataResult, PlaceCacheEntry, PlaceComponents, PlaceStatus,
};

pub const LOCATION_BUCKET_PRECISION_VERSION: u32 = 1;
pub const PLACE_CACHE_VERSION: u32 = 1;

const CACHE_DIR_NAME: &str = "cache";
const PLACE_CACHE_FILE_NAME: &str = "place-cache.json";
const CITY_BUCKET_STEP_DEGREES: f64 = 0.1;
const NOMINATIM_BASE_URL: &str = "https://nominatim.openstreetmap.org/reverse";
const NOMINATIM_PROVIDER_NAME: &str = "nominatim";
const NOMINATIM_MIN_INTERVAL: Duration = Duration::from_millis(1_100);
const NOMINATIM_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Debug, Clone)]
pub struct PlaceLookupOptions {
    pub enable_online_lookup: bool,
}

impl Default for PlaceLookupOptions {
    fn default() -> Self {
        Self {
            enable_online_lookup: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReverseGeocodeRequestMetadata {
    pub bucket_id: String,
    pub requested_at: String,
    pub user_agent: String,
}

#[derive(Debug, Clone)]
pub struct ReverseGeocodeResult {
    pub components: PlaceComponents,
}

pub trait ReverseGeocodeProvider {
    fn provider_name(&self) -> &str;
    fn user_agent(&self) -> &str;
    fn reverse_geocode(
        &self,
        coordinate: &GpsCoordinate,
        metadata: &ReverseGeocodeRequestMetadata,
    ) -> Result<Option<ReverseGeocodeResult>, String>;
}

pub trait LookupThrottle {
    fn wait_turn(&mut self) -> Result<(), String>;
}

pub struct FixedIntervalThrottle {
    min_interval: Duration,
    next_allowed_at: Option<Instant>,
}

impl FixedIntervalThrottle {
    pub fn new(min_interval: Duration) -> Self {
        Self {
            min_interval,
            next_allowed_at: None,
        }
    }
}

impl LookupThrottle for FixedIntervalThrottle {
    fn wait_turn(&mut self) -> Result<(), String> {
        let now = Instant::now();
        if let Some(next_allowed_at) = self.next_allowed_at {
            if now < next_allowed_at {
                thread::sleep(next_allowed_at - now);
            }
        }

        self.next_allowed_at = Some(Instant::now() + self.min_interval);
        Ok(())
    }
}

pub struct NominatimReverseGeocodeProvider {
    client: reqwest::blocking::Client,
    base_url: String,
    user_agent: String,
    accept_language: String,
}

impl NominatimReverseGeocodeProvider {
    pub fn new(user_agent: String) -> Result<Self, String> {
        if user_agent.trim().is_empty() {
            return Err("Nominatim provider user agent cannot be empty.".to_string());
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(NOMINATIM_TIMEOUT)
            .build()
            .map_err(|error| format!("Failed to build Nominatim HTTP client: {error}"))?;

        Ok(Self {
            client,
            base_url: NOMINATIM_BASE_URL.to_string(),
            user_agent,
            accept_language: "en".to_string(),
        })
    }

    pub fn default_user_agent() -> String {
        format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }
}

impl ReverseGeocodeProvider for NominatimReverseGeocodeProvider {
    fn provider_name(&self) -> &str {
        NOMINATIM_PROVIDER_NAME
    }

    fn user_agent(&self) -> &str {
        self.user_agent.as_str()
    }

    fn reverse_geocode(
        &self,
        coordinate: &GpsCoordinate,
        _metadata: &ReverseGeocodeRequestMetadata,
    ) -> Result<Option<ReverseGeocodeResult>, String> {
        let response = self
            .client
            .get(&self.base_url)
            .query(&[
                ("lat", coordinate.latitude.to_string()),
                ("lon", coordinate.longitude.to_string()),
                ("format", "jsonv2".to_string()),
                ("addressdetails", "1".to_string()),
                ("zoom", "10".to_string()),
            ])
            .header(reqwest::header::USER_AGENT, self.user_agent.as_str())
            .header(
                reqwest::header::ACCEPT_LANGUAGE,
                self.accept_language.as_str(),
            )
            .send()
            .map_err(|error| format!("Place lookup request failed: {error}"))?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err("Place lookup provider rate limit was reached.".to_string());
        }

        let response = response
            .error_for_status()
            .map_err(|error| format!("Place lookup provider returned HTTP error: {error}"))?;

        let parsed: NominatimReverseResponse = response
            .json()
            .map_err(|error| format!("Place lookup provider returned invalid JSON: {error}"))?;

        Ok(map_nominatim_response_to_result(parsed))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct NominatimAddress {
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
    hamlet: Option<String>,
    municipality: Option<String>,
    locality: Option<String>,
    neighbourhood: Option<String>,
    suburb: Option<String>,
    city_district: Option<String>,
    county: Option<String>,
    state_district: Option<String>,
    state: Option<String>,
    region: Option<String>,
    country: Option<String>,
    national_park: Option<String>,
    protected_area: Option<String>,
    tourism: Option<String>,
    attraction: Option<String>,
    leisure: Option<String>,
    amenity: Option<String>,
    natural: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct NominatimReverseResponse {
    address: Option<NominatimAddress>,
    display_name: Option<String>,
    name: Option<String>,
}

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
    let mut noop_throttle = NoopLookupThrottle;
    apply_place_resolution_with_provider(
        app_data_dir,
        results,
        &PlaceLookupOptions {
            enable_online_lookup: false,
        },
        &NoopReverseGeocodeProvider,
        &mut noop_throttle,
    )
}

pub fn resolve_places_for_metadata_results(
    app_data_dir: &Path,
    results: &mut [MetadataResult],
    options: &PlaceLookupOptions,
) -> Result<(), String> {
    let provider = NominatimReverseGeocodeProvider::new(
        NominatimReverseGeocodeProvider::default_user_agent(),
    )?;
    let mut throttle = FixedIntervalThrottle::new(NOMINATIM_MIN_INTERVAL);

    apply_place_resolution_with_provider(app_data_dir, results, options, &provider, &mut throttle)
}

pub fn apply_place_resolution_with_provider(
    app_data_dir: &Path,
    results: &mut [MetadataResult],
    options: &PlaceLookupOptions,
    provider: &dyn ReverseGeocodeProvider,
    throttle: &mut dyn LookupThrottle,
) -> Result<(), String> {
    if results.is_empty() {
        return Ok(());
    }

    let mut cache = load_place_cache(app_data_dir)?;
    let mut cache_was_updated = false;
    let last_used_at = Utc::now().to_rfc3339();
    let mut uncached_buckets: HashMap<String, PendingBucketLookup> = HashMap::new();

    for (index, result) in results.iter_mut().enumerate() {
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
            apply_cache_entry_to_result(entry, result);

            if entry.last_used_at != last_used_at {
                entry.last_used_at = last_used_at.clone();
                cache_was_updated = true;
            }
        } else {
            result.place_status = PlaceStatus::Pending;
            result.place = None;
            result.place_error = None;
            uncached_buckets
                .entry(bucket.bucket_id.clone())
                .and_modify(|pending| pending.result_indices.push(index))
                .or_insert_with(|| PendingBucketLookup {
                    bucket_id: bucket.bucket_id,
                    representative_coordinate: gps_coordinate.clone(),
                    result_indices: vec![index],
                });
        }
    }

    if options.enable_online_lookup {
        for pending in uncached_buckets.into_values() {
            throttle.wait_turn()?;

            let metadata = ReverseGeocodeRequestMetadata {
                bucket_id: pending.bucket_id.clone(),
                requested_at: Utc::now().to_rfc3339(),
                user_agent: provider.user_agent().to_string(),
            };

            match provider.reverse_geocode(&pending.representative_coordinate, &metadata) {
                Ok(Some(lookup)) => {
                    if let Some(display_text) = place_components_to_display_text(&lookup.components)
                    {
                        for index in pending.result_indices {
                            let result = &mut results[index];
                            result.place_status = PlaceStatus::Resolved;
                            result.place = Some(display_text.clone());
                            result.place_error = None;
                        }

                        cache.insert(
                            pending.bucket_id.clone(),
                            PlaceCacheEntry {
                                version: PLACE_CACHE_VERSION,
                                bucket_id: pending.bucket_id,
                                place: display_text,
                                components: Some(lookup.components),
                                provider: provider.provider_name().to_string(),
                                resolved_at: metadata.requested_at.clone(),
                                last_used_at: metadata.requested_at,
                            },
                        );
                        cache_was_updated = true;
                    } else {
                        for index in pending.result_indices {
                            let result = &mut results[index];
                            result.place_status = PlaceStatus::Failed;
                            result.place = None;
                            result.place_error = Some(
                                "Place lookup returned no usable city/region/country values."
                                    .to_string(),
                            );
                        }
                    }
                }
                Ok(None) => {
                    for index in pending.result_indices {
                        let result = &mut results[index];
                        result.place_status = PlaceStatus::Failed;
                        result.place = None;
                        result.place_error =
                            Some("Place lookup returned no address result.".to_string());
                    }
                }
                Err(error) => {
                    for index in pending.result_indices {
                        let result = &mut results[index];
                        result.place_status = PlaceStatus::Failed;
                        result.place = None;
                        result.place_error = Some(error.clone());
                    }
                }
            }
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

fn map_nominatim_response_to_result(
    response: NominatimReverseResponse,
) -> Option<ReverseGeocodeResult> {
    let fallback_components = response
        .display_name
        .as_deref()
        .and_then(place_components_from_display_name)
        .or_else(|| {
            response.name.as_deref().and_then(|name| {
                first_present_string(&[Some(name.to_string())]).map(|city| PlaceComponents {
                    city: Some(city),
                    region: None,
                    country: None,
                })
            })
        });

    let components = response
        .address
        .map(map_nominatim_address_to_components)
        .and_then(|components| {
            if place_components_to_display_text(&components).is_some() {
                Some(components)
            } else {
                fallback_components.clone()
            }
        })
        .or(fallback_components)?;

    Some(ReverseGeocodeResult { components })
}

fn map_nominatim_address_to_components(address: NominatimAddress) -> PlaceComponents {
    PlaceComponents {
        city: first_present_string(&[
            address.city,
            address.town,
            address.village,
            address.municipality,
            address.hamlet,
            address.locality,
            address.neighbourhood,
            address.suburb,
            address.city_district,
            address.national_park,
            address.protected_area,
            address.tourism,
            address.attraction,
            address.leisure,
            address.amenity,
            address.natural,
        ]),
        region: first_present_string(&[
            address.state,
            address.region,
            address.state_district,
            address.county,
        ]),
        country: first_present_string(&[address.country]),
    }
}

fn place_components_from_display_name(display_name: &str) -> Option<PlaceComponents> {
    let mut segments: Vec<String> = Vec::new();
    for segment in display_name.split(',') {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !segments
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(trimmed))
        {
            segments.push(trimmed.to_string());
        }
    }

    if segments.is_empty() {
        return None;
    }

    let city = segments.first().cloned();
    let country = segments.last().filter(|_| segments.len() > 1).cloned();
    let region = if segments.len() > 2 {
        segments.get(segments.len() - 2).cloned()
    } else {
        None
    };

    Some(PlaceComponents {
        city,
        region,
        country,
    })
}

pub fn place_components_to_display_text(components: &PlaceComponents) -> Option<String> {
    let mut segments: Vec<String> = Vec::new();

    for value in [
        components.city.as_deref(),
        components.region.as_deref(),
        components.country.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !segments
            .iter()
            .any(|segment| segment.eq_ignore_ascii_case(trimmed))
        {
            segments.push(trimmed.to_string());
        }
    }

    if segments.is_empty() {
        None
    } else {
        Some(segments.join(", "))
    }
}

fn first_present_string(values: &[Option<String>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn apply_cache_entry_to_result(entry: &PlaceCacheEntry, result: &mut MetadataResult) {
    result.place_status = PlaceStatus::Resolved;
    result.place = Some(entry.place.clone());
    result.place_error = None;
}

#[derive(Debug)]
struct PendingBucketLookup {
    bucket_id: String,
    representative_coordinate: GpsCoordinate,
    result_indices: Vec<usize>,
}

struct NoopReverseGeocodeProvider;

impl ReverseGeocodeProvider for NoopReverseGeocodeProvider {
    fn provider_name(&self) -> &str {
        "none"
    }

    fn user_agent(&self) -> &str {
        "none"
    }

    fn reverse_geocode(
        &self,
        _coordinate: &GpsCoordinate,
        _metadata: &ReverseGeocodeRequestMetadata,
    ) -> Result<Option<ReverseGeocodeResult>, String> {
        Ok(None)
    }
}

struct NoopLookupThrottle;

impl LookupThrottle for NoopLookupThrottle {
    fn wait_turn(&mut self) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        fs,
        sync::{Arc, Mutex},
    };

    use tempfile::tempdir;

    use super::{
        apply_cache_to_metadata_results, apply_place_resolution_with_provider,
        bucket_for_coordinate, load_place_cache, map_nominatim_response_to_result,
        place_cache_path, place_components_to_display_text, save_place_cache, LookupThrottle,
        NominatimAddress, NominatimReverseResponse, PlaceLookupOptions, ReverseGeocodeProvider,
        ReverseGeocodeRequestMetadata, ReverseGeocodeResult, LOCATION_BUCKET_PRECISION_VERSION,
        PLACE_CACHE_VERSION,
    };
    use crate::models::{
        GpsCoordinate, MetadataResult, MetadataStatus, PlaceCacheEntry, PlaceComponents,
        PlaceStatus,
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

    #[test]
    fn resolves_uncached_bucket_once_and_applies_to_all_results() {
        let temp = tempdir().expect("temp dir");
        let coordinate_a = GpsCoordinate {
            latitude: -33.8688,
            longitude: 151.2093,
        };
        let coordinate_b = GpsCoordinate {
            latitude: -33.8612,
            longitude: 151.2041,
        };

        let mut results = vec![
            metadata_result_with_coordinate("file-1", coordinate_a),
            metadata_result_with_coordinate("file-2", coordinate_b),
        ];

        let provider = MockProvider::with_results(vec![Ok(Some(ReverseGeocodeResult {
            components: PlaceComponents {
                city: Some("Sydney".to_string()),
                region: Some("NSW".to_string()),
                country: Some("Australia".to_string()),
            },
        }))]);
        let mut throttle = MockThrottle::default();

        apply_place_resolution_with_provider(
            temp.path(),
            &mut results,
            &PlaceLookupOptions {
                enable_online_lookup: true,
            },
            &provider,
            &mut throttle,
        )
        .expect("lookup should resolve");

        assert_eq!(provider.call_count(), 1);
        let calls = provider.recorded_calls();
        assert_eq!(calls[0].user_agent, "mock-user-agent");
        assert!(!calls[0].bucket_id.trim().is_empty());
        assert_eq!(throttle.call_count, 1);
        assert_eq!(results[0].place_status, PlaceStatus::Resolved);
        assert_eq!(results[1].place_status, PlaceStatus::Resolved);
        assert_eq!(results[0].place.as_deref(), Some("Sydney, NSW, Australia"));
        assert_eq!(results[1].place.as_deref(), Some("Sydney, NSW, Australia"));
        assert!(results[0].place_error.is_none());
        assert!(results[1].place_error.is_none());

        let cache = load_place_cache(temp.path()).expect("cache should load");
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn uses_first_source_coordinate_for_uncached_bucket_lookup() {
        let temp = tempdir().expect("temp dir");
        let coordinate = GpsCoordinate {
            latitude: 37.723511,
            longitude: -119.718558,
        };
        let bucket = bucket_for_coordinate(&coordinate).expect("bucket should exist");
        assert_ne!(bucket.representative_coordinate, coordinate);

        let mut results = vec![metadata_result_with_coordinate(
            "file-1",
            coordinate.clone(),
        )];
        let provider = MockProvider::with_results(vec![Ok(Some(ReverseGeocodeResult {
            components: PlaceComponents {
                city: Some("Yosemite Valley".to_string()),
                region: Some("California".to_string()),
                country: Some("United States".to_string()),
            },
        }))]);
        let mut throttle = MockThrottle::default();

        apply_place_resolution_with_provider(
            temp.path(),
            &mut results,
            &PlaceLookupOptions {
                enable_online_lookup: true,
            },
            &provider,
            &mut throttle,
        )
        .expect("lookup should resolve");

        assert_eq!(provider.recorded_coordinates(), vec![coordinate]);
        assert_eq!(
            results[0].place.as_deref(),
            Some("Yosemite Valley, California, United States")
        );
    }

    #[test]
    fn marks_bucket_failed_when_provider_returns_error() {
        let temp = tempdir().expect("temp dir");
        let coordinate = GpsCoordinate {
            latitude: -33.8688,
            longitude: 151.2093,
        };
        let mut results = vec![metadata_result_with_coordinate("file-1", coordinate)];

        let provider = MockProvider::with_results(vec![Err("provider unavailable".to_string())]);
        let mut throttle = MockThrottle::default();

        apply_place_resolution_with_provider(
            temp.path(),
            &mut results,
            &PlaceLookupOptions {
                enable_online_lookup: true,
            },
            &provider,
            &mut throttle,
        )
        .expect("resolution should keep going");

        assert_eq!(results[0].place_status, PlaceStatus::Failed);
        assert!(results[0].place.is_none());
        assert_eq!(
            results[0].place_error.as_deref(),
            Some("provider unavailable")
        );
    }

    #[test]
    fn marks_bucket_failed_when_provider_returns_empty_result() {
        let temp = tempdir().expect("temp dir");
        let coordinate = GpsCoordinate {
            latitude: -33.8688,
            longitude: 151.2093,
        };
        let mut results = vec![metadata_result_with_coordinate("file-1", coordinate)];

        let provider = MockProvider::with_results(vec![Ok(None)]);
        let mut throttle = MockThrottle::default();

        apply_place_resolution_with_provider(
            temp.path(),
            &mut results,
            &PlaceLookupOptions {
                enable_online_lookup: true,
            },
            &provider,
            &mut throttle,
        )
        .expect("resolution should keep going");

        assert_eq!(results[0].place_status, PlaceStatus::Failed);
        assert_eq!(
            results[0].place_error.as_deref(),
            Some("Place lookup returned no address result.")
        );
    }

    #[test]
    fn keeps_uncached_results_pending_when_online_lookup_disabled() {
        let temp = tempdir().expect("temp dir");
        let coordinate = GpsCoordinate {
            latitude: -33.8688,
            longitude: 151.2093,
        };
        let mut results = vec![metadata_result_with_coordinate("file-1", coordinate)];

        let provider = MockProvider::with_results(vec![Ok(Some(ReverseGeocodeResult {
            components: PlaceComponents {
                city: Some("Sydney".to_string()),
                region: Some("NSW".to_string()),
                country: Some("Australia".to_string()),
            },
        }))]);
        let mut throttle = MockThrottle::default();

        apply_place_resolution_with_provider(
            temp.path(),
            &mut results,
            &PlaceLookupOptions {
                enable_online_lookup: false,
            },
            &provider,
            &mut throttle,
        )
        .expect("resolution should apply cached entries");

        assert_eq!(provider.call_count(), 0);
        assert_eq!(throttle.call_count, 0);
        assert_eq!(results[0].place_status, PlaceStatus::Pending);
        assert!(results[0].place.is_none());
        assert!(results[0].place_error.is_none());
    }

    #[test]
    fn returns_error_when_throttle_blocks_lookup() {
        let temp = tempdir().expect("temp dir");
        let coordinate = GpsCoordinate {
            latitude: -33.8688,
            longitude: 151.2093,
        };
        let mut results = vec![metadata_result_with_coordinate("file-1", coordinate)];

        let provider = MockProvider::with_results(vec![Ok(Some(ReverseGeocodeResult {
            components: PlaceComponents {
                city: Some("Sydney".to_string()),
                region: Some("NSW".to_string()),
                country: Some("Australia".to_string()),
            },
        }))]);
        let mut throttle = MockThrottle {
            call_count: 0,
            error: Some("throttled".to_string()),
        };

        let error = apply_place_resolution_with_provider(
            temp.path(),
            &mut results,
            &PlaceLookupOptions {
                enable_online_lookup: true,
            },
            &provider,
            &mut throttle,
        )
        .expect_err("throttle error should surface");

        assert_eq!(error, "throttled");
        assert_eq!(provider.call_count(), 0);
    }

    #[test]
    fn builds_city_level_display_text_from_components() {
        let place = place_components_to_display_text(&PlaceComponents {
            city: Some("Sydney".to_string()),
            region: Some("New South Wales".to_string()),
            country: Some("Australia".to_string()),
        });
        assert_eq!(place.as_deref(), Some("Sydney, New South Wales, Australia"));
    }

    #[test]
    fn falls_back_to_region_and_country_when_city_is_missing() {
        let place = place_components_to_display_text(&PlaceComponents {
            city: None,
            region: Some("Queensland".to_string()),
            country: Some("Australia".to_string()),
        });
        assert_eq!(place.as_deref(), Some("Queensland, Australia"));
    }

    #[test]
    fn builds_place_from_sparse_nominatim_display_name() {
        let result = map_nominatim_response_to_result(NominatimReverseResponse {
            address: None,
            display_name: Some(
                "Yosemite Valley, Mariposa County, California, United States".to_string(),
            ),
            name: None,
        })
        .expect("display name should provide a fallback place");

        assert_eq!(
            place_components_to_display_text(&result.components).as_deref(),
            Some("Yosemite Valley, California, United States")
        );
    }

    #[test]
    fn builds_place_from_park_address_fields() {
        let result = map_nominatim_response_to_result(NominatimReverseResponse {
            address: Some(NominatimAddress {
                national_park: Some("Yosemite National Park".to_string()),
                state: Some("California".to_string()),
                country: Some("United States".to_string()),
                ..Default::default()
            }),
            display_name: None,
            name: None,
        })
        .expect("park address should provide a fallback place");

        assert_eq!(
            place_components_to_display_text(&result.components).as_deref(),
            Some("Yosemite National Park, California, United States")
        );
    }

    #[derive(Debug)]
    struct MockProvider {
        responses: Arc<Mutex<Vec<Result<Option<ReverseGeocodeResult>, String>>>>,
        calls: Arc<Mutex<Vec<ReverseGeocodeRequestMetadata>>>,
        coordinates: Arc<Mutex<Vec<GpsCoordinate>>>,
    }

    impl MockProvider {
        fn with_results(results: Vec<Result<Option<ReverseGeocodeResult>, String>>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(results)),
                calls: Arc::new(Mutex::new(Vec::new())),
                coordinates: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn call_count(&self) -> usize {
            self.calls.lock().expect("calls lock").len()
        }

        fn recorded_calls(&self) -> Vec<ReverseGeocodeRequestMetadata> {
            self.calls.lock().expect("calls lock").clone()
        }

        fn recorded_coordinates(&self) -> Vec<GpsCoordinate> {
            self.coordinates.lock().expect("coordinates lock").clone()
        }
    }

    impl ReverseGeocodeProvider for MockProvider {
        fn provider_name(&self) -> &str {
            "mock"
        }

        fn user_agent(&self) -> &str {
            "mock-user-agent"
        }

        fn reverse_geocode(
            &self,
            coordinate: &GpsCoordinate,
            metadata: &ReverseGeocodeRequestMetadata,
        ) -> Result<Option<ReverseGeocodeResult>, String> {
            self.calls
                .lock()
                .expect("calls lock")
                .push(metadata.clone());
            self.coordinates
                .lock()
                .expect("coordinates lock")
                .push(coordinate.clone());

            let mut responses = self.responses.lock().expect("responses lock");
            if responses.is_empty() {
                return Ok(None);
            }

            responses.remove(0)
        }
    }

    #[derive(Debug, Default)]
    struct MockThrottle {
        call_count: usize,
        error: Option<String>,
    }

    impl LookupThrottle for MockThrottle {
        fn wait_turn(&mut self) -> Result<(), String> {
            self.call_count += 1;
            if let Some(error) = self.error.as_ref() {
                return Err(error.clone());
            }
            Ok(())
        }
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
