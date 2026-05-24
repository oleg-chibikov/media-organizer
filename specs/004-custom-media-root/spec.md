# Feature Specification: Custom Root Media Folder

**Feature Branch**: `004-custom-media-root`  
**Created**: 2026-05-24  
**Status**: Draft  
**Input**: User wants the app to use a configurable root Media folder instead of the selected folder's `Processed` directory, persist that choice across runs, expose folder selection from a right-side menu, and provide a toggle that forces legacy `Processed` behavior even when a custom root is configured.

## User Scenarios & Testing

### Primary User Story

As a user organizing NAS media, I want to choose a dedicated root Media folder where processed files are stored so the app can move files from a TODO folder into my actual media library instead of always creating or using a `Processed` folder beside the TODO source.

### Acceptance Scenarios

1. **Choose custom media root**
   - Given the app is open
   - When the user opens the right-side settings menu and chooses a root Media folder
   - Then the app stores that absolute folder path as the configured root Media folder
   - And the selected root Media folder remains visible in the menu

2. **Persist custom media root**
   - Given the user configured a root Media folder
   - When the app is closed and opened again
   - Then the same root Media folder is loaded automatically
   - And future move planning uses that folder unless legacy `Processed` mode is forced

3. **Plan destinations under custom media root**
   - Given a TODO folder is selected and a root Media folder is configured
   - And the force `Processed` toggle is off
   - When files are scanned and move plans are created
   - Then destination directories are created under the configured root Media folder using the existing date-based folder structure
   - And file naming, conflict handling, metadata date rules, and history recording remain unchanged

4. **Execute moves under custom media root**
   - Given a move plan points to the configured root Media folder
   - When the user confirms processing
   - Then move safety validation allows destinations inside the configured root Media folder
   - And rejects destinations outside the effective destination root

5. **Force legacy Processed folder**
   - Given a root Media folder is configured
   - When the user enables the force `Processed` toggle
   - Then all planning and execution use `<selected TODO folder>/Processed`
   - And the configured root Media folder remains persisted but inactive

6. **Recover from unavailable custom root**
   - Given a persisted root Media folder no longer exists or is unreadable
   - When the app starts or planning begins
   - Then the app shows a clear settings error
   - And move planning/execution does not silently fall back to a different destination unless the user enables force `Processed`

### Edge Cases

- No root Media folder is configured.
- A persisted root Media folder was deleted, renamed, disconnected, or is unreadable.
- The configured root Media folder is the same as the selected TODO folder.
- The configured root Media folder is inside the selected TODO folder.
- The selected TODO folder is inside the configured root Media folder.
- The root Media folder contains a path segment named `Processed`.
- The selected TODO folder is inside a `Processed` path while force `Processed` is enabled.
- The user clears the configured root Media folder.
- The user changes the configured root Media folder while planning or processing is already running.
- The app cannot read or write appData settings.
- Existing history entries reference destinations from both legacy `Processed` and custom media roots.
- Undo/redo runs created with a custom media root after the configured root changes.

## Requirements

### Functional Requirements

- **FR-001**: The app shall persist destination settings in the app's appData directory.
- **FR-002**: Destination settings shall include an optional absolute `root_media_folder` path and a boolean `force_processed_folder` value.
- **FR-003**: On startup, the frontend shall load destination settings before move planning begins.
- **FR-004**: The app shall expose Tauri commands to get destination settings, set or clear the root Media folder, and set the force `Processed` toggle.
- **FR-005**: The app shall provide a right-side settings menu that shows the current root Media folder, lets the user choose a folder, lets the user clear it, and exposes the force `Processed` toggle.
- **FR-006**: Choosing a root Media folder shall use the platform folder picker and persist the selected absolute folder path.
- **FR-007**: If `root_media_folder` is set and `force_processed_folder` is false, move planning shall place destination directories under `root_media_folder`.
- **FR-008**: If `root_media_folder` is not set, or `force_processed_folder` is true, move planning shall preserve current behavior and place destination directories under `<selected folder>/Processed`.
- **FR-009**: The existing date-based destination directory structure under the effective destination root shall remain unchanged.
- **FR-010**: The existing generated filename format, conflict suffix behavior, metadata selection, streaming planning behavior, and move result states shall remain unchanged.
- **FR-011**: Move safety validation shall validate destinations against the effective destination root instead of always validating against `<selected folder>/Processed`.
- **FR-012**: Move safety validation shall reject source files that are inside the effective destination root to prevent re-processing already organized files.
- **FR-013**: Folder scanning shall continue to reject selected TODO folders inside `Processed` paths for legacy behavior, but shall not reject the custom root Media folder only because its path contains `Processed`.
- **FR-014**: The app shall reject move planning and execution when the configured root Media folder is missing, not a directory, or unreadable and force `Processed` is false.
- **FR-015**: The app shall not silently fall back from a configured root Media folder to `<selected folder>/Processed` when the configured root is invalid.
- **FR-016**: The app shall preserve a configured root Media folder when force `Processed` is toggled on or off.
- **FR-017**: The app shall disable or block root/fallback setting changes while move execution is active, or apply changes only to future planning batches with a clear UI state.
- **FR-018**: Processing history shall record the effective destination root mode and effective destination root path used for each run.
- **FR-019**: Undo and redo shall use recorded operation paths and shall not depend on the current root Media folder setting.
- **FR-020**: Tests shall cover persisted settings, custom-root planning, forced legacy planning, move safety validation, and frontend settings controls.

### Non-Functional Requirements

- **NFR-001**: Settings reads and writes shall be small appData JSON operations and shall not block long-running file processing.
- **NFR-002**: Settings files shall be versioned so incompatible future settings can be ignored or migrated safely.
- **NFR-003**: Destination root resolution shall be deterministic and shared by planning and execution to avoid plan/execute mismatches.
- **NFR-004**: Error messages shall distinguish between "custom root unavailable" and "legacy Processed folder forced".
- **NFR-005**: The right-side settings menu shall remain usable on desktop and narrow layouts.

## Data Model

### DestinationSettings

- `version`: Settings schema version.
- `root_media_folder`: Optional absolute folder path.
- `force_processed_folder`: Whether to use `<selected folder>/Processed` even when `root_media_folder` is set.
- `updated_at`: Timestamp when settings last changed.

### EffectiveDestinationRoot

- `mode`: `custom_root` or `processed_folder`.
- `root_path`: Absolute destination root path used by planning and safety validation.
- `custom_root_configured`: Whether a root Media folder is persisted.
- `force_processed_folder`: Whether legacy mode is forced.
- `warning`: Optional user-visible validation warning.

### HistoryRun Additions

- `destination_root_mode`: `custom_root` or `processed_folder`.
- `destination_root_path`: Absolute destination root path used for the run.

## Success Criteria

- **SC-001**: A selected root Media folder remains configured after app restart.
- **SC-002**: With a valid root Media folder and force `Processed` off, planned destinations are under the configured root Media folder.
- **SC-003**: With force `Processed` on, planned destinations are under `<selected folder>/Processed` even if a root Media folder is configured.
- **SC-004**: Move execution accepts only destinations under the effective destination root for the current run.
- **SC-005**: Invalid or unavailable custom root folders block planning/execution with a clear error.
- **SC-006**: Existing undo/redo remains path-based and works for runs created before and after this feature.

## Out Of Scope For First Milestone

- Multiple named media roots.
- Per-file or per-media-type destination roots.
- Migrating existing files from `Processed` to the root Media folder.
- Auto-detecting a media root from NAS metadata.
- Syncing settings across devices.

## Resolved Decisions

- The configured root Media folder replaces only the destination root, not the selected TODO/source folder.
- Force `Processed` is a persistent user setting that preserves the configured root Media folder while temporarily bypassing it.
- Planning and execution must both receive or resolve the same effective destination root.
- Undo/redo remains based on recorded operation paths and does not recalculate paths using current settings.
