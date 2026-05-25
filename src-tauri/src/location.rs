use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{
    GpsCoordinate, LocationBucket, MetadataResult, PlaceCacheEntry, PlaceComponents,
    PlaceLookupDiagnosticResult, PlaceStatus,
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
const SPATIAL_INDEX_CELL_DEGREES: f64 = 1.0;

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
    pub area: Option<ResolvedPlaceArea>,
}

#[derive(Debug, Clone)]
pub struct ResolvedPlaceArea {
    pub center: GpsCoordinate,
    pub radius_km: f64,
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

    pub fn diagnostic_lookup(
        &self,
        coordinate: &GpsCoordinate,
    ) -> Result<PlaceLookupDiagnosticResult, String> {
        validate_coordinate(coordinate)?;

        let request_url = self.build_reverse_url(coordinate)?.to_string();
        let bucket = bucket_for_coordinate(coordinate);
        let response = self
            .client
            .get(&request_url)
            .header(reqwest::header::USER_AGENT, self.user_agent.as_str())
            .header(
                reqwest::header::ACCEPT_LANGUAGE,
                self.accept_language.as_str(),
            )
            .send()
            .map_err(|error| format!("Place lookup request failed: {error}"))?;
        let http_status = response.status();

        if http_status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Ok(PlaceLookupDiagnosticResult {
                coordinate: coordinate.clone(),
                bucket_id: bucket.map(|value| value.bucket_id),
                provider: NOMINATIM_PROVIDER_NAME.to_string(),
                request_url,
                http_status: Some(http_status.as_u16()),
                display_place: None,
                components: None,
                raw_response: None,
                error: Some("Place lookup provider rate limit was reached.".to_string()),
            });
        }

        let response = response
            .error_for_status()
            .map_err(|error| format!("Place lookup provider returned HTTP error: {error}"))?;
        let raw_response: Value = response
            .json()
            .map_err(|error| format!("Place lookup provider returned invalid JSON: {error}"))?;
        let parsed: NominatimReverseResponse = serde_json::from_value(raw_response.clone())
            .map_err(|error| {
                format!("Place lookup provider response shape was invalid: {error}")
            })?;
        let result = map_nominatim_response_to_result(parsed);
        let (display_place, components, error) = match result {
            Some(result) => {
                let display_place = place_components_to_display_text(&result.components);
                (display_place, Some(result.components), None)
            }
            None => (
                None,
                None,
                Some("Place lookup returned no address or display-name result.".to_string()),
            ),
        };

        Ok(PlaceLookupDiagnosticResult {
            coordinate: coordinate.clone(),
            bucket_id: bucket.map(|value| value.bucket_id),
            provider: NOMINATIM_PROVIDER_NAME.to_string(),
            request_url,
            http_status: Some(http_status.as_u16()),
            display_place,
            components,
            raw_response: Some(raw_response),
            error,
        })
    }

    fn build_reverse_url(&self, coordinate: &GpsCoordinate) -> Result<reqwest::Url, String> {
        reqwest::Url::parse_with_params(
            &self.base_url,
            &[
                ("lat", coordinate.latitude.to_string()),
                ("lon", coordinate.longitude.to_string()),
                ("format", "jsonv2".to_string()),
                ("addressdetails", "1".to_string()),
                ("zoom", "10".to_string()),
            ],
        )
        .map_err(|error| format!("Failed to build place lookup URL: {error}"))
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
    #[serde(default)]
    boundingbox: Option<Vec<String>>,
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

    let raw = match fs::read_to_string(&cache_path) {
        Ok(value) => value,
        Err(error) => {
            return Err(format!(
                "Failed to read place cache file at {}: {error}",
                cache_path.display()
            ));
        }
    };

    let cache_file: PlaceCacheFile = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(error) => {
            quarantine_corrupt_cache_file(&cache_path);
            eprintln!(
                "Place cache parse failed at {}: {error}. Falling back to empty cache.",
                cache_path.display()
            );
            return Ok(HashMap::new());
        }
    };

    if cache_file.version != PLACE_CACHE_VERSION {
        return Ok(HashMap::new());
    }

    let mut entries = HashMap::new();
    for mut entry in cache_file.entries {
        if entry.version != PLACE_CACHE_VERSION || entry.bucket_id.trim().is_empty() {
            continue;
        }
        if entry
            .area_radius_km
            .is_some_and(|value| !value.is_finite() || value <= 0.0)
        {
            entry.area_radius_km = None;
        }
        entry.version = PLACE_CACHE_VERSION;
        entries.insert(entry.bucket_id.clone(), entry);
    }

    Ok(entries)
}

fn quarantine_corrupt_cache_file(cache_path: &Path) {
    let backup_path = cache_path.with_extension("corrupt");
    let _ = fs::rename(cache_path, backup_path);
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
    let mut spatial_index = AreaSpatialIndex::from_cache(&cache);
    let mut cache_was_updated = false;
    let last_used_at = Utc::now().to_rfc3339();
    let mut queue: VecDeque<PendingLocationLookup> = VecDeque::new();

    for (index, result) in results.iter_mut().enumerate() {
        let Some(gps_coordinate) = result.gps_coordinate.as_ref() else {
            result.place_bucket_id = None;
            result.place_status = PlaceStatus::NotAvailable;
            result.place = None;
            result.place_error = None;
            continue;
        };

        let Some(area_id) = area_id_for_coordinate(gps_coordinate) else {
            result.place_bucket_id = None;
            result.place_status = PlaceStatus::NotAvailable;
            result.place = None;
            result.place_error = None;
            continue;
        };

        result.place_bucket_id = Some(area_id.clone());
        if let Some(entry_id) =
            find_covering_cache_entry_id(gps_coordinate, &cache, &spatial_index, Some(&area_id))
        {
            let entry = cache
                .get_mut(&entry_id)
                .expect("cache entry id from index should exist");
            apply_cache_entry_to_result(entry, result);
            result.place_bucket_id = Some(entry_id.clone());

            if entry.last_used_at != last_used_at {
                entry.last_used_at = last_used_at.clone();
                cache_was_updated = true;
            }
        } else {
            result.place_status = PlaceStatus::Pending;
            result.place = None;
            result.place_error = None;
            queue.push_back(PendingLocationLookup {
                area_id,
                coordinate: gps_coordinate.clone(),
                result_index: index,
            });
        }
    }

    if options.enable_online_lookup {
        while let Some(pending) = queue.pop_front() {
            let result = &mut results[pending.result_index];
            if result.gps_coordinate.is_none() {
                continue;
            }

            if let Some(entry_id) = find_covering_cache_entry_id(
                &pending.coordinate,
                &cache,
                &spatial_index,
                Some(&pending.area_id),
            ) {
                let entry = cache
                    .get_mut(&entry_id)
                    .expect("cache entry id from index should exist");
                apply_cache_entry_to_result(entry, result);
                result.place_bucket_id = Some(entry_id.clone());

                if entry.last_used_at != last_used_at {
                    entry.last_used_at = last_used_at.clone();
                    cache_was_updated = true;
                }
                continue;
            }

            throttle.wait_turn()?;

            let metadata = ReverseGeocodeRequestMetadata {
                bucket_id: pending.area_id.clone(),
                requested_at: Utc::now().to_rfc3339(),
                user_agent: provider.user_agent().to_string(),
            };

            match provider.reverse_geocode(&pending.coordinate, &metadata) {
                Ok(Some(lookup)) => {
                    if let Some(display_text) = place_components_to_display_text(&lookup.components)
                    {
                        let area_center = lookup.area.as_ref().map(|value| value.center.clone());
                        let area_radius_km = lookup.area.as_ref().map(|value| value.radius_km);
                        let area_entry = PlaceCacheEntry {
                            version: PLACE_CACHE_VERSION,
                            bucket_id: pending.area_id.clone(),
                            place: display_text.clone(),
                            components: Some(lookup.components),
                            provider: provider.provider_name().to_string(),
                            resolved_at: metadata.requested_at.clone(),
                            last_used_at: metadata.requested_at.clone(),
                            area_center,
                            area_radius_km,
                        };
                        cache.insert(pending.area_id.clone(), area_entry);
                        if let Some(entry) = cache.get(&pending.area_id) {
                            spatial_index.insert_entry(entry);
                        }

                        result.place_status = PlaceStatus::Resolved;
                        result.place = Some(display_text);
                        result.place_error = None;
                        result.place_bucket_id = Some(pending.area_id.clone());
                        cache_was_updated = true;
                    } else {
                        result.place_status = PlaceStatus::Failed;
                        result.place = None;
                        result.place_error = Some(
                            "Place lookup returned no usable city/region/country values."
                                .to_string(),
                        );
                    }
                }
                Ok(None) => {
                    result.place_status = PlaceStatus::Failed;
                    result.place = None;
                    result.place_error =
                        Some("Place lookup returned no address result.".to_string());
                }
                Err(error) => {
                    result.place_status = PlaceStatus::Failed;
                    result.place = None;
                    result.place_error = Some(error);
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

fn validate_coordinate(coordinate: &GpsCoordinate) -> Result<(), String> {
    if is_valid_coordinate(coordinate) {
        Ok(())
    } else {
        Err("GPS coordinates are not valid latitude/longitude values.".to_string())
    }
}

fn quantize_coordinate(value: f64) -> i32 {
    (value / CITY_BUCKET_STEP_DEGREES).floor() as i32
}

fn bucket_center_from_index(index: i32, min: f64, max: f64) -> f64 {
    ((index as f64 + 0.5) * CITY_BUCKET_STEP_DEGREES).clamp(min, max)
}

fn area_id_for_coordinate(coordinate: &GpsCoordinate) -> Option<String> {
    bucket_for_coordinate(coordinate).map(|bucket| bucket.bucket_id)
}

fn find_covering_cache_entry_id(
    coordinate: &GpsCoordinate,
    cache: &HashMap<String, PlaceCacheEntry>,
    index: &AreaSpatialIndex,
    preferred_area_id: Option<&str>,
) -> Option<String> {
    if let Some(area_id) = preferred_area_id {
        if let Some(entry) = cache.get(area_id) {
            if cache_entry_contains_coordinate(entry, coordinate, preferred_area_id) {
                return Some(area_id.to_string());
            }
        }
    }

    for entry_id in index.lookup(coordinate) {
        if let Some(entry) = cache.get(entry_id) {
            if cache_entry_contains_coordinate(entry, coordinate, preferred_area_id) {
                return Some(entry_id.to_string());
            }
        }
    }

    for (entry_id, entry) in cache {
        if cache_entry_contains_coordinate(entry, coordinate, preferred_area_id) {
            return Some(entry_id.clone());
        }
    }

    None
}

fn cache_entry_contains_coordinate(
    entry: &PlaceCacheEntry,
    coordinate: &GpsCoordinate,
    preferred_area_id: Option<&str>,
) -> bool {
    if let (Some(center), Some(radius_km)) = (entry.area_center.as_ref(), entry.area_radius_km) {
        if radius_km.is_finite() && radius_km > 0.0 {
            return haversine_distance_km(center, coordinate) <= radius_km;
        }
    }

    let Some(area_id) = preferred_area_id else {
        return false;
    };
    entry.bucket_id == area_id
}

fn haversine_distance_km(a: &GpsCoordinate, b: &GpsCoordinate) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6_371.0;
    let lat1 = a.latitude.to_radians();
    let lat2 = b.latitude.to_radians();
    let d_lat = (b.latitude - a.latitude).to_radians();
    let d_lon = normalize_longitude(b.longitude - a.longitude).to_radians();

    let sin_d_lat = (d_lat / 2.0).sin();
    let sin_d_lon = (d_lon / 2.0).sin();
    let haversine = sin_d_lat * sin_d_lat + lat1.cos() * lat2.cos() * sin_d_lon * sin_d_lon;
    let arc = 2.0 * haversine.sqrt().atan2((1.0 - haversine).sqrt());
    EARTH_RADIUS_KM * arc
}

fn normalize_longitude(value: f64) -> f64 {
    (value + 180.0).rem_euclid(360.0) - 180.0
}

fn map_nominatim_response_to_result(
    response: NominatimReverseResponse,
) -> Option<ReverseGeocodeResult> {
    let area = response
        .boundingbox
        .as_deref()
        .and_then(parse_nominatim_bounding_box);
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

    Some(ReverseGeocodeResult { components, area })
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

fn parse_nominatim_bounding_box(values: &[String]) -> Option<ResolvedPlaceArea> {
    if values.len() != 4 {
        return None;
    }

    let south = values.first()?.parse::<f64>().ok()?;
    let north = values.get(1)?.parse::<f64>().ok()?;
    let west = values.get(2)?.parse::<f64>().ok()?;
    let east = values.get(3)?.parse::<f64>().ok()?;

    if !south.is_finite()
        || !north.is_finite()
        || !west.is_finite()
        || !east.is_finite()
        || !(-90.0..=90.0).contains(&south)
        || !(-90.0..=90.0).contains(&north)
        || !(-180.0..=180.0).contains(&west)
        || !(-180.0..=180.0).contains(&east)
        || south > north
    {
        return None;
    }

    let east_unwrapped = if east < west { east + 360.0 } else { east };
    let center = GpsCoordinate {
        latitude: (south + north) / 2.0,
        longitude: normalize_longitude((west + east_unwrapped) / 2.0),
    };

    let corners = [
        GpsCoordinate {
            latitude: south,
            longitude: west,
        },
        GpsCoordinate {
            latitude: south,
            longitude: normalize_longitude(east_unwrapped),
        },
        GpsCoordinate {
            latitude: north,
            longitude: west,
        },
        GpsCoordinate {
            latitude: north,
            longitude: normalize_longitude(east_unwrapped),
        },
    ];
    let radius_km = corners
        .iter()
        .map(|corner| haversine_distance_km(&center, corner))
        .fold(0.0, f64::max);

    if !radius_km.is_finite() || radius_km <= 0.0 {
        return None;
    }

    Some(ResolvedPlaceArea { center, radius_km })
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
struct PendingLocationLookup {
    area_id: String,
    coordinate: GpsCoordinate,
    result_index: usize,
}

#[derive(Debug, Default)]
struct AreaSpatialIndex {
    entries_by_cell: HashMap<String, Vec<String>>,
}

impl AreaSpatialIndex {
    fn from_cache(cache: &HashMap<String, PlaceCacheEntry>) -> Self {
        let mut index = Self::default();
        for entry in cache.values() {
            index.insert_entry(entry);
        }
        index
    }

    fn lookup<'a>(&'a self, coordinate: &GpsCoordinate) -> impl Iterator<Item = &'a str> + 'a {
        let key = spatial_cell_key(coordinate.latitude, coordinate.longitude);
        self.entries_by_cell
            .get(&key)
            .into_iter()
            .flat_map(|values| values.iter().map(String::as_str))
    }

    fn insert_entry(&mut self, entry: &PlaceCacheEntry) {
        let Some(center) = entry.area_center.as_ref() else {
            return;
        };
        let Some(radius_km) = entry.area_radius_km else {
            return;
        };
        if !radius_km.is_finite() || radius_km <= 0.0 {
            return;
        }

        let (min_lat, max_lat, min_lon, max_lon) = bounding_box_for_circle(center, radius_km);
        let min_lat_index = quantize_by_step(min_lat, SPATIAL_INDEX_CELL_DEGREES);
        let max_lat_index = quantize_by_step(max_lat, SPATIAL_INDEX_CELL_DEGREES);
        let min_lon_index = quantize_by_step(min_lon, SPATIAL_INDEX_CELL_DEGREES);
        let max_lon_index = quantize_by_step(max_lon, SPATIAL_INDEX_CELL_DEGREES);

        for lat_index in min_lat_index..=max_lat_index {
            for lon_index in min_lon_index..=max_lon_index {
                let key = format!("{lat_index}:{lon_index}");
                let entries = self.entries_by_cell.entry(key).or_default();
                if !entries.iter().any(|value| value == &entry.bucket_id) {
                    entries.push(entry.bucket_id.clone());
                }
            }
        }
    }
}

fn spatial_cell_key(latitude: f64, longitude: f64) -> String {
    let lat_index = quantize_by_step(latitude, SPATIAL_INDEX_CELL_DEGREES);
    let lon_index = quantize_by_step(longitude, SPATIAL_INDEX_CELL_DEGREES);
    format!("{lat_index}:{lon_index}")
}

fn quantize_by_step(value: f64, step: f64) -> i32 {
    (value / step).floor() as i32
}

fn bounding_box_for_circle(center: &GpsCoordinate, radius_km: f64) -> (f64, f64, f64, f64) {
    let latitude_delta = radius_km / 111.0;
    let center_lat_radians = center.latitude.to_radians();
    let cosine = center_lat_radians.cos().abs().max(0.01);
    let longitude_delta = radius_km / (111.0 * cosine);

    let min_lat = (center.latitude - latitude_delta).clamp(-90.0, 90.0);
    let max_lat = (center.latitude + latitude_delta).clamp(-90.0, 90.0);
    let min_lon = (center.longitude - longitude_delta).clamp(-180.0, 180.0);
    let max_lon = (center.longitude + longitude_delta).clamp(-180.0, 180.0);
    (min_lat, max_lat, min_lon, max_lon)
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
        NominatimAddress, NominatimReverseResponse, PlaceLookupOptions, ResolvedPlaceArea,
        ReverseGeocodeProvider, ReverseGeocodeRequestMetadata, ReverseGeocodeResult,
        LOCATION_BUCKET_PRECISION_VERSION, PLACE_CACHE_VERSION,
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
                area_center: Some(GpsCoordinate {
                    latitude: -33.8688,
                    longitude: 151.2093,
                }),
                area_radius_km: Some(50.0),
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
    fn treats_corrupt_place_cache_as_empty() {
        let temp = tempdir().expect("temp dir");
        let cache_path = place_cache_path(temp.path());
        fs::create_dir_all(cache_path.parent().expect("cache parent should exist"))
            .expect("create cache directory");
        fs::write(&cache_path, "{invalid json").expect("write corrupt cache");

        let loaded = load_place_cache(temp.path()).expect("corrupt cache should not fail");
        assert!(loaded.is_empty());
        assert!(!cache_path.exists());
        assert!(cache_path.with_extension("corrupt").exists());
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
                area_center: Some(coordinate.clone()),
                area_radius_km: Some(50.0),
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
            area: None,
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
    fn stores_provider_area_in_cache_for_uncached_bucket() {
        let temp = tempdir().expect("temp dir");
        let coordinate = GpsCoordinate {
            latitude: 37.723511,
            longitude: -119.718558,
        };
        let bucket = bucket_for_coordinate(&coordinate).expect("bucket should exist");
        let expected_center = GpsCoordinate {
            latitude: 37.75,
            longitude: -119.70,
        };
        let expected_radius_km = 12.5;
        let mut results = vec![metadata_result_with_coordinate("file-1", coordinate)];
        let provider = MockProvider::with_results(vec![Ok(Some(ReverseGeocodeResult {
            components: PlaceComponents {
                city: Some("Yosemite Valley".to_string()),
                region: Some("California".to_string()),
                country: Some("United States".to_string()),
            },
            area: Some(ResolvedPlaceArea {
                center: expected_center.clone(),
                radius_km: expected_radius_km,
            }),
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

        let cache = load_place_cache(temp.path()).expect("cache should load");
        let entry = cache
            .get(&bucket.bucket_id)
            .expect("bucket entry should be cached");
        assert_eq!(entry.area_center.as_ref(), Some(&expected_center));
        assert_eq!(entry.area_radius_km, Some(expected_radius_km));
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
            area: None,
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
            area: None,
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
            area: None,
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
            boundingbox: None,
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
            boundingbox: None,
        })
        .expect("park address should provide a fallback place");

        assert_eq!(
            place_components_to_display_text(&result.components).as_deref(),
            Some("Yosemite National Park, California, United States")
        );
    }

    #[test]
    fn maps_nominatim_bounding_box_to_area() {
        let result = map_nominatim_response_to_result(NominatimReverseResponse {
            address: Some(NominatimAddress {
                city: Some("Yosemite Valley".to_string()),
                state: Some("California".to_string()),
                country: Some("United States".to_string()),
                ..Default::default()
            }),
            display_name: None,
            name: None,
            boundingbox: Some(vec![
                "37.7000".to_string(),
                "37.8000".to_string(),
                "-119.8000".to_string(),
                "-119.6000".to_string(),
            ]),
        })
        .expect("place should resolve");

        let area = result.area.expect("bounding box should map to area");
        assert!((area.center.latitude - 37.75).abs() < 0.00001);
        assert!((area.center.longitude + 119.7).abs() < 0.00001);
        assert!(area.radius_km > 0.0);
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
