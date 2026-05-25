# Feature Specification: Human-Readable Media Place

**Feature Branch**: `005-human-readable-place`  
**Created**: 2026-05-25  
**Status**: Draft  
**Input**: User wants metadata scanning to detect photo/video GPS location, resolve a human-readable place such as a city, display it next to each media file, and avoid calling web APIs once per photo by grouping nearby files into shared locations and using only free lookup resources.

## User Scenarios & Testing

### Primary User Story

As a user organizing photos and videos, I want the app to show the city or nearest useful place where each media file was taken so I can understand the contents of a scan without manually opening files or inspecting raw coordinates.

### Acceptance Scenarios

1. **Read GPS metadata**
   - Given a photo or video contains GPS metadata
   - When metadata scanning completes for that file
   - Then the app extracts latitude and longitude in normalized decimal degrees
   - And the row becomes eligible for place resolution

2. **Display place per media row**
   - Given a media file has resolved place information
   - When the file appears in the table
   - Then the row shows a human-readable place value near the existing metadata/date columns
   - And city-level text is sufficient, such as `Sydney, NSW, Australia`

3. **Group nearby files before lookup**
   - Given many files were taken in the same city or nearby area
   - When place resolution runs
   - Then the app assigns them to a shared location bucket
   - And performs at most one external lookup for that bucket when the place is not already cached

4. **Reuse cached places**
   - Given a location bucket was resolved during a previous scan
   - When another scan contains files in the same bucket
   - Then the app uses the cached place name
   - And does not perform another web lookup for that bucket

5. **Handle missing or unusable location**
   - Given a media file has no GPS metadata or invalid coordinates
   - When metadata scanning completes
   - Then the app shows a clear empty or unavailable place state
   - And the file remains otherwise processable

6. **Recover when lookup is unavailable**
   - Given coordinates were extracted but no free lookup resource is reachable or allowed
   - When place resolution fails
   - Then the app keeps the raw location bucket pending or failed with a clear status
   - And metadata date scanning, planning, and moving continue without location text

### Edge Cases

- GPS metadata exists in photo EXIF, video QuickTime/XMP tags, or sidecar-like metadata exposed by ExifTool.
- Coordinates are zero, out of range, malformed, rounded, or missing latitude/longitude hemisphere fields.
- Multiple files in the same city have slightly different coordinates.
- Files are near a city boundary or in a rural location where no city is available.
- A lookup returns suburb, town, county, state, or country but no city field.
- The free lookup provider rate-limits, times out, returns no result, or changes response shape.
- The user scans thousands of files from many locations.
- The same cached bucket should work after app restart.
- Users may not want coordinates sent to any network service.

## Requirements

### Functional Requirements

- **FR-001**: The backend shall extract GPS latitude and longitude during metadata scanning for supported photos and videos when coordinates are present.
- **FR-002**: The backend shall normalize valid coordinates to decimal degrees and reject values outside latitude `-90..90` and longitude `-180..180`.
- **FR-003**: Metadata results shall include optional GPS coordinates and a location status independent of capture-date status.
- **FR-004**: The backend shall group coordinates into deterministic location buckets before any external place lookup.
- **FR-005**: The bucket precision shall target city-level grouping, not address-level accuracy.
- **FR-006**: The app shall perform no more than one external place lookup per uncached location bucket during a scan.
- **FR-007**: The app shall persist resolved location buckets in appData so future scans can reuse place names without network access.
- **FR-008**: The app shall use only free lookup resources for online reverse geocoding and shall respect provider usage limits.
- **FR-009**: The app shall not require a paid API key for this feature.
- **FR-010**: The app shall expose lookup failures as recoverable per-bucket or per-file statuses without blocking metadata scanning, planning, moving, undo, or redo.
- **FR-011**: Place display text shall prefer city/town/village-level names, then fall back to a useful administrative region and country when no city-like value exists.
- **FR-012**: The frontend shall display the resolved place next to each media file in the table.
- **FR-013**: The frontend shall distinguish `pending`, `resolved`, `unavailable`, and `failed` location states without treating them as move blockers.
- **FR-014**: The app shall avoid exposing raw GPS coordinates in the primary table by default; detailed coordinates may be shown only in an inspection/detail surface.
- **FR-015**: The backend shall support disabling online lookups while still extracting GPS metadata and using existing cached place values.
- **FR-016**: The app shall throttle and bound concurrent online place lookups.
- **FR-017**: Tests shall cover GPS parsing, coordinate validation, bucket grouping, cache reuse, lookup failure handling, and frontend display states.

### Non-Functional Requirements

- **NFR-001**: Place resolution shall not slow metadata extraction for files that do not contain GPS data.
- **NFR-002**: Network lookup behavior shall be deterministic enough for tests through an injectable lookup provider or mockable adapter.
- **NFR-003**: The cache format shall be versioned so bucket precision or provider changes can be migrated safely.
- **NFR-004**: The feature shall protect privacy by sending only one representative coordinate per location bucket, never every file coordinate.
- **NFR-005**: The UI shall remain responsive while place resolution is pending.

## Data Model

### MetadataResult Additions

- `gps_coordinate`: Optional normalized coordinate with `latitude` and `longitude`.
- `gps_source`: Optional metadata field or source family used to read the coordinate.
- `place_bucket_id`: Optional deterministic bucket identifier assigned from the coordinate.
- `place_status`: `not_available`, `pending`, `resolved`, or `failed`.
- `place`: Optional resolved human-readable place.
- `place_error`: Optional recoverable error message.

### LocationBucket

- `bucket_id`: Stable identifier derived from city-level coordinate bucketing.
- `representative_coordinate`: Latitude/longitude used for lookup.
- `precision`: Bucket precision version.
- `media_count`: Number of current scan files assigned to the bucket.

### PlaceCacheEntry

- `version`: Cache schema version.
- `bucket_id`: Location bucket identifier.
- `place`: Human-readable display value.
- `components`: Optional structured values such as city, state/region, and country.
- `provider`: Lookup source used.
- `resolved_at`: Timestamp when the value was resolved.
- `last_used_at`: Timestamp when the cache entry was last reused.

## Success Criteria

- **SC-001**: Media files with valid GPS metadata show a city-level place when a cached or online lookup succeeds.
- **SC-002**: A scan containing 100 files from the same location performs at most one online lookup for that location bucket.
- **SC-003**: Re-scanning files from a cached location performs no online lookup for that bucket.
- **SC-004**: Missing GPS data and failed place lookups do not block move planning or execution.
- **SC-005**: The primary table shows human-readable place names without exposing raw coordinates by default.

## Out Of Scope For First Milestone

- Address-level reverse geocoding.
- Paid geocoding providers or required API keys.
- Map views, pins, or route visualization.
- Editing GPS metadata.
- Using place names in destination folder or filename generation.
- Moving sidecar files based on location.

## Resolved Decisions

- City-level place text is enough for the first milestone.
- The Rust backend owns coordinate extraction, bucketing, cache lookup, and online lookup orchestration.
- The frontend mirrors backend location statuses and does not call geocoding services directly.
- Place lookup is supplemental metadata and must not affect date selection, destination planning, move safety, history, undo, or redo.
