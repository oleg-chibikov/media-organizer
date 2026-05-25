# Tasks: Human-Readable Media Place

**Spec**: `specs/005-human-readable-place/spec.md`  
**Plan**: `specs/005-human-readable-place/plan.md`

## Phase 1: Shared Models

- [x] T001 Add Rust models for GPS coordinate, place status, location bucket, and place cache entry.
- [x] T002 Add TypeScript payload types for GPS and place fields.
- [x] T003 Extend metadata result serialization with optional GPS and place fields while preserving compatibility for missing fields.

## Phase 2: GPS Metadata Extraction

- [x] T004 Identify ExifTool GPS fields used by supported photos and videos.
- [x] T005 Parse decimal and hemisphere-based latitude/longitude values from ExifTool output.
- [x] T006 Validate coordinate ranges and mark malformed or missing GPS as location unavailable.
- [x] T007 Add Rust tests for valid, missing, malformed, zero, and out-of-range coordinates.

## Phase 3: Location Bucketing And Cache

- [ ] T008 Add deterministic city-level bucket generation for valid coordinates.
- [ ] T009 Add versioned appData place cache read/write behavior.
- [ ] T010 Reuse cached place entries before attempting any online lookup.
- [ ] T011 Add Rust tests for bucket stability, nearby grouping, distant separation, and cache reuse.

## Phase 4: Place Lookup Provider

- [ ] T012 Add a provider abstraction for reverse geocoding lookups.
- [ ] T013 Implement a free no-paid-key provider adapter with required throttling and request metadata.
- [ ] T014 Ensure only one online lookup is attempted per uncached bucket per scan.
- [ ] T015 Convert provider response components into city-level display text.
- [ ] T016 Support disabling online lookup while still using cached place entries.
- [ ] T017 Add Rust tests with mocked provider responses, failures, empty results, and rate-limit behavior.

## Phase 5: Backend Integration

- [ ] T018 Integrate GPS extraction into the existing metadata scan flow.
- [ ] T019 Integrate bucket assignment and cache-first place resolution for GPS-bearing results.
- [ ] T020 Emit metadata or location updates so every file in a resolved bucket receives the same place result.
- [ ] T021 Keep place lookup failures non-blocking for planning, moving, history, undo, and redo.
- [ ] T022 Add targeted integration tests for scan results with resolved, unavailable, pending, and failed place states.

## Phase 6: Frontend Display

- [ ] T023 Update API bindings to receive GPS and place fields.
- [ ] T024 Add a place display to each media row near existing metadata columns.
- [ ] T025 Render pending, resolved, unavailable, and failed location states.
- [ ] T026 Keep raw coordinates hidden from the primary table by default.
- [ ] T027 Add frontend tests for place display and non-blocking failed lookup behavior.

## Phase 7: Validation

- [ ] T028 Run `mise x -- cargo test --manifest-path src-tauri/Cargo.toml`.
- [ ] T029 Run `mise x -- pnpm test`.
- [ ] T030 Run `mise x -- pnpm lint`.
- [ ] T031 Run `mise x -- pnpm build`.
- [ ] T032 Smoke-test a folder containing media from at least two location buckets and repeated files from one bucket.
