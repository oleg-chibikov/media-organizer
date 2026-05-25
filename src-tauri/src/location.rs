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
    GpsCoordinate, MetadataResult, PlaceCacheEntry, PlaceComponents, PlaceLookupDiagnosticResult,
    PlaceStatus,
};

pub const PLACE_CACHE_VERSION: u32 = 3;

const CACHE_DIR_NAME: &str = "cache";
const PLACE_CACHE_FILE_NAME: &str = "place-cache.json";
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
    pub display_name: Option<String>,
    pub components: PlaceComponents,
    pub polygon_geojson: Option<String>,
    pub osm_place_key: Option<String>,
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
                cached: false,
                osm_place_key: None,
                provider: NOMINATIM_PROVIDER_NAME.to_string(),
                request_url,
                http_status: Some(http_status.as_u16()),
                display_place: None,
                components: None,
                polygon_geojson: None,
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
        let (display_place, components, osm_place_key, polygon_geojson, error) = match result {
            Some(result) => {
                let display_place = place_components_to_display_text(&result.components);
                (
                    display_place,
                    Some(result.components),
                    result.osm_place_key,
                    result.polygon_geojson,
                    None,
                )
            }
            None => (
                None,
                None,
                None,
                None,
                Some("Place lookup returned no address or display-name result.".to_string()),
            ),
        };

        Ok(PlaceLookupDiagnosticResult {
            coordinate: coordinate.clone(),
            cached: false,
            osm_place_key,
            provider: NOMINATIM_PROVIDER_NAME.to_string(),
            request_url,
            http_status: Some(http_status.as_u16()),
            display_place,
            components,
            polygon_geojson,
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
                ("zoom", "14".to_string()),
                ("polygon_geojson", "1".to_string()),
            ],
        )
        .map_err(|error| format!("Failed to build place lookup URL: {error}"))
    }
}

pub fn diagnostic_lookup_with_cache(
    app_data_dir: &Path,
    coordinate: &GpsCoordinate,
) -> Result<PlaceLookupDiagnosticResult, String> {
    validate_coordinate(coordinate)?;

    let provider = NominatimReverseGeocodeProvider::new(
        NominatimReverseGeocodeProvider::default_user_agent(),
    )?;
    let request_url = provider.build_reverse_url(coordinate)?.to_string();
    let cache = load_place_cache(app_data_dir)?;

    if let Some((entry_key, entry)) = find_covering_cache_entry(coordinate, &cache) {
        return Ok(PlaceLookupDiagnosticResult {
            coordinate: coordinate.clone(),
            cached: true,
            osm_place_key: Some(entry_key.clone()),
            provider: entry.provider.clone(),
            request_url,
            http_status: None,
            display_place: Some(entry.place.clone()),
            components: entry.components.clone(),
            polygon_geojson: entry.polygon_geojson.clone(),
            raw_response: Some(serde_json::json!({
                "source": "cache",
                "cache_entry": entry,
            })),
            error: None,
        });
    }

    provider.diagnostic_lookup(coordinate)
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
                ("zoom", "14".to_string()),
                ("polygon_geojson", "1".to_string()),
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
    place_id: Option<u64>,
    osm_type: Option<String>,
    osm_id: Option<i64>,
    address: Option<NominatimAddress>,
    display_name: Option<String>,
    name: Option<String>,
    #[serde(default)]
    geojson: Option<Value>,
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
        if entry.version != PLACE_CACHE_VERSION || entry.osm_place_key.trim().is_empty() {
            continue;
        }
        entry.version = PLACE_CACHE_VERSION;
        entries.insert(entry.osm_place_key.clone(), entry);
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
    ordered_entries.sort_by(|left, right| left.osm_place_key.cmp(&right.osm_place_key));
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
    let mut queue: VecDeque<PendingLocationLookup> = VecDeque::new();

    for (index, result) in results.iter_mut().enumerate() {
        let Some(gps_coordinate) = result.gps_coordinate.as_ref() else {
            result.place_bucket_id = None;
            result.place_status = PlaceStatus::NotAvailable;
            result.place = None;
            result.place_error = None;
            continue;
        };

        if !is_valid_coordinate(gps_coordinate) {
            result.place_bucket_id = None;
            result.place_status = PlaceStatus::NotAvailable;
            result.place = None;
            result.place_error = None;
            continue;
        }

        if let Some((entry_id, entry)) = find_covering_cache_entry(gps_coordinate, &cache) {
            let entry_id = entry_id.clone();
            apply_cache_entry_to_result(entry, result);
            result.place_bucket_id = Some(entry_id.clone());

            if let Some(entry) = cache.get_mut(&entry_id) {
                if entry.last_used_at != last_used_at {
                    entry.last_used_at = last_used_at.clone();
                    cache_was_updated = true;
                }
            }
        } else {
            result.place_status = PlaceStatus::Pending;
            result.place = None;
            result.place_error = None;
            queue.push_back(PendingLocationLookup {
                result_index: index,
            });
        }
    }

    if options.enable_online_lookup {
        while let Some(pending) = queue.pop_front() {
            let result = &mut results[pending.result_index];
            let Some(gps_coordinate) = result.gps_coordinate.as_ref() else {
                continue;
            };

            if let Some((entry_id, entry)) = find_covering_cache_entry(gps_coordinate, &cache) {
                let entry_id = entry_id.clone();
                apply_cache_entry_to_result(entry, result);
                result.place_bucket_id = Some(entry_id.clone());

                if let Some(entry) = cache.get_mut(&entry_id) {
                    if entry.last_used_at != last_used_at {
                        entry.last_used_at = last_used_at.clone();
                        cache_was_updated = true;
                    }
                }
                continue;
            }

            throttle.wait_turn()?;

            let metadata = ReverseGeocodeRequestMetadata {
                bucket_id: "".to_string(),
                requested_at: Utc::now().to_rfc3339(),
                user_agent: provider.user_agent().to_string(),
            };

            match provider.reverse_geocode(gps_coordinate, &metadata) {
                Ok(Some(lookup)) => {
                    let display_text = lookup
                        .display_name
                        .filter(|name| !name.trim().is_empty())
                        .or_else(|| place_components_to_display_text(&lookup.components));

                    if let Some(display_text) = display_text {
                        let osm_place_key = lookup
                            .osm_place_key
                            .unwrap_or_else(|| format!("unknown:{}", gps_coordinate.latitude));
                        let area_entry = PlaceCacheEntry {
                            version: PLACE_CACHE_VERSION,
                            osm_place_key: osm_place_key.clone(),
                            place: display_text.clone(),
                            components: Some(lookup.components),
                            provider: provider.provider_name().to_string(),
                            resolved_at: metadata.requested_at.clone(),
                            last_used_at: metadata.requested_at.clone(),
                            polygon_geojson: lookup.polygon_geojson,
                        };
                        cache.insert(osm_place_key.clone(), area_entry);

                        result.place_status = PlaceStatus::Resolved;
                        result.place = Some(display_text);
                        result.place_error = None;
                        result.place_bucket_id = Some(osm_place_key);
                        cache_was_updated = true;
                    } else {
                        result.place_status = PlaceStatus::Failed;
                        result.place = None;
                        result.place_error = Some(
                            "Place lookup returned no usable address or city/region/country values."
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

fn find_covering_cache_entry<'a>(
    coordinate: &GpsCoordinate,
    cache: &'a HashMap<String, PlaceCacheEntry>,
) -> Option<(&'a String, &'a PlaceCacheEntry)> {
    let mut best: Option<(&String, &PlaceCacheEntry, f64)> = None;

    for (key, entry) in cache {
        let Some(ref geojson_str) = entry.polygon_geojson else {
            continue;
        };
        let Ok(geojson) = serde_json::from_str::<Value>(geojson_str) else {
            continue;
        };
        if !point_in_geojson_polygon(coordinate, &geojson) {
            continue;
        }
        let area = geojson_estimated_area(&geojson);
        match best {
            Some((_, _, best_area)) if area < best_area => {
                best = Some((key, entry, area));
            }
            None => {
                best = Some((key, entry, area));
            }
            _ => {}
        }
    }

    best.map(|(key, entry, _)| (key, entry))
}

fn point_in_geojson_polygon(point: &GpsCoordinate, geojson: &Value) -> bool {
    let geom_type = geojson.get("type").and_then(Value::as_str);
    match geom_type {
        Some("Polygon") => {
            let coords = geojson.get("coordinates").and_then(Value::as_array);
            point_in_polygon_coords(point, coords)
        }
        Some("MultiPolygon") => {
            let polygons = geojson.get("coordinates").and_then(Value::as_array);
            if let Some(polygons) = polygons {
                for polygon_coords in polygons {
                    if let Some(coords) = polygon_coords.as_array() {
                        if point_in_polygon_coords(point, Some(coords)) {
                            return true;
                        }
                    }
                }
            }
            false
        }
        _ => false,
    }
}

fn point_in_polygon_coords(point: &GpsCoordinate, rings: Option<&Vec<Value>>) -> bool {
    let Some(rings) = rings else { return false };
    let Some(exterior) = rings.first().and_then(|r| r.as_array()) else {
        return false;
    };
    if !point_in_ring(point, exterior) {
        return false;
    }
    for ring in rings.iter().skip(1) {
        if let Some(ring) = ring.as_array() {
            if point_in_ring(point, ring) {
                return false;
            }
        }
    }
    true
}

fn point_in_ring(point: &GpsCoordinate, ring: &[Value]) -> bool {
    let mut inside = false;
    let mut j = ring.len() - 1;

    for i in 0..ring.len() {
        let vi_lon = ring[i].get(0).and_then(Value::as_f64);
        let vi_lat = ring[i].get(1).and_then(Value::as_f64);
        let vj_lon = ring[j].get(0).and_then(Value::as_f64);
        let vj_lat = ring[j].get(1).and_then(Value::as_f64);

        if let (Some(vi_lon), Some(vi_lat), Some(vj_lon), Some(vj_lat)) =
            (vi_lon, vi_lat, vj_lon, vj_lat)
        {
            if ((vi_lat > point.latitude) != (vj_lat > point.latitude))
                && point.longitude
                    < (vj_lon - vi_lon) * (point.latitude - vi_lat) / (vj_lat - vi_lat) + vi_lon
            {
                inside = !inside;
            }
        }

        j = i;
    }

    inside
}

fn geojson_estimated_area(geojson: &Value) -> f64 {
    let coords = geojson.get("coordinates").and_then(Value::as_array);
    match geojson.get("type").and_then(Value::as_str) {
        Some("Polygon") => coords
            .and_then(|c| c.first())
            .and_then(|r| r.as_array())
            .map(|r| ring_estimated_area(r))
            .unwrap_or(f64::MAX),
        Some("MultiPolygon") => coords
            .and_then(|polys| polys.first())
            .and_then(|p| p.as_array())
            .and_then(|c| c.first())
            .and_then(|r| r.as_array())
            .map(|r| ring_estimated_area(r))
            .unwrap_or(f64::MAX),
        _ => f64::MAX,
    }
}

fn ring_estimated_area(ring: &[Value]) -> f64 {
    if ring.len() < 3 {
        return f64::MAX;
    }
    let mut area = 0.0;
    let mut j = ring.len() - 1;
    for i in 0..ring.len() {
        let xi = ring[i].get(0).and_then(Value::as_f64).unwrap_or(0.0);
        let yi = ring[i].get(1).and_then(Value::as_f64).unwrap_or(0.0);
        let xj = ring[j].get(0).and_then(Value::as_f64).unwrap_or(0.0);
        let yj = ring[j].get(1).and_then(Value::as_f64).unwrap_or(0.0);
        area += xj * yi - xi * yj;
        j = i;
    }
    area.abs() / 2.0
}

fn map_nominatim_response_to_result(
    response: NominatimReverseResponse,
) -> Option<ReverseGeocodeResult> {
    let osm_place_key = response
        .osm_type
        .as_deref()
        .zip(response.osm_id.as_ref())
        .map(|(osm_type, osm_id)| format!("osm:{osm_type}:{osm_id}"))
        .or_else(|| response.place_id.map(|id| format!("place_id:{id}")));

    let polygon_geojson = response.geojson.as_ref().and_then(|g| {
        let geom_type = g.get("type").and_then(Value::as_str)?;
        matches!(geom_type, "Polygon" | "MultiPolygon").then(|| g.to_string())
    });

    let fallback_components = response
        .display_name
        .as_deref()
        .and_then(place_components_from_display_name)
        .or_else(|| {
            response.name.as_deref().and_then(|name| {
                first_present_string(&[Some(name.to_string())]).map(|city| PlaceComponents {
                    landmark: None,
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

    Some(ReverseGeocodeResult {
        display_name: response.display_name,
        components,
        polygon_geojson,
        osm_place_key,
    })
}

fn map_nominatim_address_to_components(address: NominatimAddress) -> PlaceComponents {
    PlaceComponents {
        landmark: first_present_string(&[
            address.national_park,
            address.protected_area,
            address.tourism,
            address.attraction,
            address.leisure,
            address.amenity,
            address.natural,
        ]),
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
        landmark: None,
        city,
        region,
        country,
    })
}

pub fn place_components_to_display_text(components: &PlaceComponents) -> Option<String> {
    let mut segments: Vec<String> = Vec::new();

    for value in [
        components.landmark.as_deref(),
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
    result_index: usize,
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

    use serde_json::Value;

    use super::{
        apply_cache_to_metadata_results, apply_place_resolution_with_provider, load_place_cache,
        map_nominatim_response_to_result, place_cache_path, place_components_to_display_text,
        point_in_geojson_polygon, save_place_cache, LookupThrottle, NominatimAddress,
        NominatimReverseResponse, PlaceLookupOptions, ReverseGeocodeProvider,
        ReverseGeocodeRequestMetadata, ReverseGeocodeResult, PLACE_CACHE_VERSION,
    };
    use crate::models::{
        GpsCoordinate, MetadataResult, MetadataStatus, PlaceCacheEntry, PlaceComponents,
        PlaceStatus,
    };

    #[test]
    fn persists_versioned_place_cache() {
        let temp = tempdir().expect("temp dir");
        let mut entries = HashMap::new();
        entries.insert(
            "osm:relation:12345".to_string(),
            PlaceCacheEntry {
                version: 99,
                osm_place_key: "osm:relation:12345".to_string(),
                place: "Sydney, NSW, Australia".to_string(),
                components: None,
                provider: "cache-test".to_string(),
                resolved_at: "2026-05-25T00:00:00Z".to_string(),
                last_used_at: "2026-05-25T00:00:00Z".to_string(),
                polygon_geojson: None,
            },
        );

        save_place_cache(temp.path(), &entries).expect("cache should save");
        let loaded = load_place_cache(temp.path()).expect("cache should load");
        let persisted = loaded
            .get("osm:relation:12345")
            .expect("entry should round-trip with same osm_place_key");

        assert_eq!(persisted.version, PLACE_CACHE_VERSION);
        assert!(persisted.polygon_geojson.is_none());

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

        let mut entries = HashMap::new();
        let sydney_polygon = r#"{"type":"Polygon","coordinates":[[[151.0,-34.0],[152.0,-34.0],[152.0,-33.0],[151.0,-33.0],[151.0,-34.0]]]}"#;
        entries.insert(
            "osm:relation:12345".to_string(),
            PlaceCacheEntry {
                version: PLACE_CACHE_VERSION,
                osm_place_key: "osm:relation:12345".to_string(),
                place: "Sydney, NSW, Australia".to_string(),
                components: None,
                provider: "nominatim".to_string(),
                resolved_at: "2026-05-25T00:00:00Z".to_string(),
                last_used_at: "2026-05-25T00:00:00Z".to_string(),
                polygon_geojson: Some(sydney_polygon.to_string()),
            },
        );
        save_place_cache(temp.path(), &entries).expect("cache should save");

        let mut results = vec![metadata_result_with_coordinate("file-1", coordinate)];
        apply_cache_to_metadata_results(temp.path(), &mut results).expect("cache should apply");

        let resolved = &results[0];
        assert_eq!(
            resolved.place_bucket_id.as_deref(),
            Some("osm:relation:12345")
        );
        assert_eq!(resolved.place_status, PlaceStatus::Resolved);
        assert_eq!(resolved.place.as_deref(), Some("Sydney, NSW, Australia"));
        assert!(resolved.place_error.is_none());

        let loaded = load_place_cache(temp.path()).expect("cache should reload");
        let refreshed = loaded
            .get("osm:relation:12345")
            .expect("entry should remain in cache");
        assert_ne!(refreshed.last_used_at, "2026-05-25T00:00:00Z");
    }

    #[test]
    fn resolves_uncached_place_once_and_applies_to_all_results() {
        let temp = tempdir().expect("temp dir");
        let coordinate_a = GpsCoordinate {
            latitude: -33.8688,
            longitude: 151.2093,
        };
        let coordinate_b = GpsCoordinate {
            latitude: -33.8612,
            longitude: 151.2041,
        };

        let sydney_polygon = r#"{"type":"Polygon","coordinates":[[[151.0,-34.0],[152.0,-34.0],[152.0,-33.0],[151.0,-33.0],[151.0,-34.0]]]}"#;

        let mut results = vec![
            metadata_result_with_coordinate("file-1", coordinate_a),
            metadata_result_with_coordinate("file-2", coordinate_b),
        ];

        let provider = MockProvider::with_results(vec![Ok(Some(ReverseGeocodeResult {
            display_name: None,
            components: PlaceComponents {
                landmark: None,
                city: Some("Sydney".to_string()),
                region: Some("NSW".to_string()),
                country: Some("Australia".to_string()),
            },
            polygon_geojson: Some(sydney_polygon.to_string()),
            osm_place_key: Some("osm:relation:12345".to_string()),
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
    fn caches_polygon_from_provider_response() {
        let temp = tempdir().expect("temp dir");
        let coordinate = GpsCoordinate {
            latitude: 37.75,
            longitude: -119.70,
        };
        let polygon_json = r#"{"type":"Polygon","coordinates":[[[-119.8,37.7],[-119.6,37.7],[-119.6,37.8],[-119.8,37.8],[-119.8,37.7]]]}"#;
        let mut results = vec![metadata_result_with_coordinate("file-1", coordinate)];
        let provider = MockProvider::with_results(vec![Ok(Some(ReverseGeocodeResult {
            display_name: None,
            components: PlaceComponents {
                landmark: None,
                city: Some("Yosemite Valley".to_string()),
                region: Some("California".to_string()),
                country: Some("United States".to_string()),
            },
            polygon_geojson: Some(polygon_json.to_string()),
            osm_place_key: Some("osm:way:12345".to_string()),
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
        let entry = cache.get("osm:way:12345").expect("entry should be cached");
        assert_eq!(entry.polygon_geojson.as_deref(), Some(polygon_json));
    }

    #[test]
    fn uses_source_coordinate_for_uncached_place_lookup() {
        let temp = tempdir().expect("temp dir");
        let coordinate = GpsCoordinate {
            latitude: 37.723511,
            longitude: -119.718558,
        };

        let mut results = vec![metadata_result_with_coordinate(
            "file-1",
            coordinate.clone(),
        )];
        let provider = MockProvider::with_results(vec![Ok(Some(ReverseGeocodeResult {
            display_name: None,
            components: PlaceComponents {
                landmark: None,
                city: Some("Yosemite Valley".to_string()),
                region: Some("California".to_string()),
                country: Some("United States".to_string()),
            },
            polygon_geojson: None,
            osm_place_key: None,
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
    fn marks_place_failed_when_provider_returns_error() {
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
    fn marks_place_failed_when_provider_returns_empty_result() {
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
            display_name: None,
            components: PlaceComponents {
                landmark: None,
                city: Some("Sydney".to_string()),
                region: Some("NSW".to_string()),
                country: Some("Australia".to_string()),
            },
            polygon_geojson: None,
            osm_place_key: None,
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
            display_name: None,
            components: PlaceComponents {
                landmark: None,
                city: Some("Sydney".to_string()),
                region: Some("NSW".to_string()),
                country: Some("Australia".to_string()),
            },
            polygon_geojson: None,
            osm_place_key: None,
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
            landmark: None,
            city: Some("Sydney".to_string()),
            region: Some("New South Wales".to_string()),
            country: Some("Australia".to_string()),
        });
        assert_eq!(place.as_deref(), Some("Sydney, New South Wales, Australia"));
    }

    #[test]
    fn includes_landmark_when_present_in_display_text() {
        let place = place_components_to_display_text(&PlaceComponents {
            landmark: Some("Taronga Zoo".to_string()),
            city: Some("Sydney".to_string()),
            region: Some("NSW".to_string()),
            country: Some("Australia".to_string()),
        });
        assert_eq!(
            place.as_deref(),
            Some("Taronga Zoo, Sydney, NSW, Australia")
        );
    }

    #[test]
    fn shows_landmark_as_place_when_no_city() {
        let place = place_components_to_display_text(&PlaceComponents {
            landmark: Some("Yosemite National Park".to_string()),
            city: None,
            region: Some("California".to_string()),
            country: Some("United States".to_string()),
        });
        assert_eq!(
            place.as_deref(),
            Some("Yosemite National Park, California, United States")
        );
    }

    #[test]
    fn falls_back_to_region_and_country_when_city_is_missing() {
        let place = place_components_to_display_text(&PlaceComponents {
            landmark: None,
            city: None,
            region: Some("Queensland".to_string()),
            country: Some("Australia".to_string()),
        });
        assert_eq!(place.as_deref(), Some("Queensland, Australia"));
    }

    #[test]
    fn builds_place_from_sparse_nominatim_display_name() {
        let result = map_nominatim_response_to_result(NominatimReverseResponse {
            place_id: None,
            osm_type: None,
            osm_id: None,
            address: None,
            display_name: Some(
                "Yosemite Valley, Mariposa County, California, United States".to_string(),
            ),
            name: None,
            geojson: None,
        })
        .expect("display name should provide a fallback place");

        assert_eq!(
            place_components_to_display_text(&result.components).as_deref(),
            Some("Yosemite Valley, California, United States")
        );
    }

    #[test]
    fn extracts_polygon_from_nominatim_response() {
        let polygon_json = r#"{"type":"Polygon","coordinates":[[[-119.8,37.7],[-119.6,37.7],[-119.6,37.8],[-119.8,37.8],[-119.8,37.7]]]}"#;
        let result = map_nominatim_response_to_result(NominatimReverseResponse {
            place_id: None,
            osm_type: Some("way".to_string()),
            osm_id: Some(12345),
            address: Some(NominatimAddress {
                city: Some("Yosemite Valley".to_string()),
                state: Some("California".to_string()),
                country: Some("United States".to_string()),
                national_park: Some("Yosemite National Park".to_string()),
                ..Default::default()
            }),
            display_name: None,
            name: None,
            geojson: Some(serde_json::from_str(polygon_json).expect("valid geojson")),
        })
        .expect("place should resolve");

        assert_eq!(result.osm_place_key.as_deref(), Some("osm:way:12345"));
        let roundtrip: serde_json::Value =
            serde_json::from_str(result.polygon_geojson.as_deref().unwrap()).expect("roundtrip");
        let expected: serde_json::Value = serde_json::from_str(polygon_json).expect("expected");
        assert_eq!(roundtrip, expected);
    }

    #[test]
    fn point_in_simple_polygon() {
        let polygon = serde_json::from_str::<Value>(
            r#"{"type":"Polygon","coordinates":[[[0.0,0.0],[2.0,0.0],[2.0,2.0],[0.0,2.0],[0.0,0.0]]]}"#
        ).expect("valid geojson");
        let inside = GpsCoordinate {
            latitude: 1.0,
            longitude: 1.0,
        };
        let outside = GpsCoordinate {
            latitude: 3.0,
            longitude: 3.0,
        };
        assert!(point_in_geojson_polygon(&inside, &polygon));
        assert!(!point_in_geojson_polygon(&outside, &polygon));
    }

    #[test]
    fn point_in_multi_polygon() {
        let multipolygon = serde_json::from_str::<Value>(
            r#"{"type":"MultiPolygon","coordinates":[[[[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,1.0],[0.0,0.0]]],[[[2.0,2.0],[3.0,2.0],[3.0,3.0],[2.0,3.0],[2.0,2.0]]]]}"#
        ).expect("valid geojson");
        let inside = GpsCoordinate {
            latitude: 0.5,
            longitude: 0.5,
        };
        let also_inside = GpsCoordinate {
            latitude: 2.5,
            longitude: 2.5,
        };
        let outside = GpsCoordinate {
            latitude: 1.5,
            longitude: 1.5,
        };
        assert!(point_in_geojson_polygon(&inside, &multipolygon));
        assert!(point_in_geojson_polygon(&also_inside, &multipolygon));
        assert!(!point_in_geojson_polygon(&outside, &multipolygon));
    }

    #[test]
    fn point_outside_polygon_with_hole() {
        let polygon = serde_json::from_str::<Value>(
            r#"{"type":"Polygon","coordinates":[[[0.0,0.0],[3.0,0.0],[3.0,3.0],[0.0,3.0],[0.0,0.0]],[[1.0,1.0],[2.0,1.0],[2.0,2.0],[1.0,2.0],[1.0,1.0]]]}"#
        ).expect("valid geojson");
        let in_hole = GpsCoordinate {
            latitude: 1.5,
            longitude: 1.5,
        };
        let in_polygon = GpsCoordinate {
            latitude: 0.5,
            longitude: 0.5,
        };
        assert!(!point_in_geojson_polygon(&in_hole, &polygon));
        assert!(point_in_geojson_polygon(&in_polygon, &polygon));
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
