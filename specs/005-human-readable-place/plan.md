# Implementation Plan: Human-Readable Media Place

**Spec**: `specs/005-human-readable-place/spec.md`  
**Stack**: Tauri 2, React, TypeScript, Rust  
**Milestone Target**: Extract GPS metadata, group nearby coordinates, resolve city-level place names through cache-first free lookup, and display place status per media row.

## Architecture

Location handling belongs in the Rust backend because ExifTool parsing, appData cache persistence, network throttling, and privacy boundaries must be consistent and testable.

React receives location fields through the existing metadata result flow, renders place text and status in the file table, and does not call geocoding services directly.

The first milestone should resolve place names after GPS-bearing metadata results are available. Place resolution is supplemental: date extraction, planning, moving, history, undo, and redo must continue when place lookup is unavailable.

## AppData Layout

Use the app data directory for a small versioned cache.

```text
appData/
  cache/
    place-cache.json
```

The cache stores one entry per location bucket, not one entry per media file.

## Backend Changes

Add or extend Rust modules:

- `models.rs`: add GPS coordinate, place status, location bucket, and place cache DTOs.
- `metadata.rs`: parse GPS fields from ExifTool output for photos and videos and include optional coordinates in metadata results.
- `location.rs`: new module for coordinate validation, bucket generation, cache read/write, provider abstraction, and display-name selection.
- `commands.rs`: orchestrate cache-first place resolution for metadata results with GPS data.
- `events.rs`: extend metadata events or add location events if place resolution is asynchronous after metadata completion.
- `main.rs`: register any state or commands required by the location resolver.

## Frontend Changes

Update API and UI bindings:

- Add GPS and place fields to `src/lib/types.ts`.
- Ensure `src/lib/api.ts` event payload types match Rust metadata/location events.
- Add a place column or compact row field in `FileTable`.
- Show pending, resolved, unavailable, and failed states without treating them as move-blocking errors.
- Hide raw coordinates from the main table by default.

## Location Resolution Rules

1. Extract and validate latitude/longitude from ExifTool metadata.
2. Assign valid coordinates to deterministic city-level buckets.
3. Check the appData place cache by bucket id.
4. For uncached buckets, perform one throttled online lookup per bucket using a free provider.
5. Convert provider response components into display text, preferring city/town/village plus region and country.
6. Store successful lookups in the cache and apply the result to every file in the bucket.
7. Mark failed lookups as recoverable failures and leave media processing unaffected.

## Provider Strategy

Implement the lookup provider behind a Rust trait or adapter so tests can inject deterministic responses.

The first provider must be free and require no paid API key. The app should respect published rate limits, set a clear user agent if required, and keep concurrency low.

Online lookup should be disableable in settings or configuration. When disabled, the app still extracts GPS coordinates and uses existing cached bucket entries.

## Compatibility

Existing metadata results without GPS fields remain valid. Frontend code should treat missing location fields as `not_available`.

No existing move plan, move result, history, undo, or redo behavior should change because place names are display metadata only in the first milestone.

## Testing Strategy

Rust tests:

- parse decimal and hemisphere GPS fields from representative ExifTool output
- reject malformed and out-of-range coordinates
- assign nearby coordinates to the same bucket at chosen precision
- assign distant coordinates to different buckets
- reuse cached place entries before provider lookup
- call provider once per uncached bucket
- map provider responses to city-level display text
- handle provider timeout, rate limit, and empty result as recoverable failures

Frontend tests:

- render resolved place text in a media row
- render unavailable state for files without GPS metadata
- render pending and failed place states
- keep move eligibility unchanged when place lookup fails

## First Milestone

1. Add shared Rust and TypeScript models for GPS and place status.
2. Parse and validate GPS coordinates from ExifTool metadata.
3. Add deterministic location bucketing and appData place cache.
4. Add cache-first provider-backed place resolution with throttling.
5. Display place values and states in the file table.
6. Validate with targeted Rust and frontend tests, then broader checks when practical.

## Risks

- City-level bucketing may group nearby border locations incorrectly. Mitigation: keep precision versioned and display provider-derived region/country to reduce ambiguity.
- Free providers may rate-limit or become unavailable. Mitigation: cache aggressively, throttle requests, and make lookup failure non-blocking.
- Sending coordinates to a service has privacy implications. Mitigation: send one representative coordinate per bucket, support disabling online lookup, and never send every file coordinate.
