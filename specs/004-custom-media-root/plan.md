# Implementation Plan: Custom Root Media Folder

**Spec**: `specs/004-custom-media-root/spec.md`  
**Stack**: Tauri 2, React, TypeScript, Rust  
**Milestone Target**: Persisted destination settings plus planning/execution against a custom root Media folder with a force-Processed fallback

## Architecture

Destination settings are owned by the Rust backend because appData path resolution, folder validation, and move safety checks must be consistent and testable.

React owns the right-side settings menu, displays the configured root Media folder, and passes the effective settings through planning and execution calls.

Planning and execution should share a single destination-root resolver so a move plan cannot be generated for one root and executed under a different safety boundary.

## AppData Layout

Use Tauri's app data directory for this app.

```text
appData/
  settings/
    destination-settings.json
```

The settings JSON is versioned and small enough to read on startup and after settings mutations.

## Backend Changes

Add or extend Rust modules:

- `settings.rs`: appData settings path resolution, default settings, read/write, and validation.
- `models.rs`: add `DestinationSettings`, `EffectiveDestinationRoot`, request/response DTOs, and history destination-root fields.
- `commands.rs`: expose destination settings commands and thread effective destination root through planning and execution.
- `planner.rs`: accept an effective destination root instead of hard-coding `<selected>/Processed`.
- `mover.rs`: validate source and destination paths against the effective destination root instead of hard-coding `<selected>/Processed`.
- `history.rs`: record destination root mode/path for new runs while preserving compatibility with existing history files.
- `main.rs`: register the new commands and module.

## Frontend Changes

Add a right-side settings menu to the main app shell:

- Load destination settings on startup.
- Show the configured root Media folder path or "Not set".
- Provide "Choose Media Root" using a new backend command.
- Provide "Clear Media Root".
- Provide a persistent force `Processed` toggle.
- Disable or clearly defer settings changes while processing is active.
- Surface root validation errors near the menu and in existing folder/move error surfaces.

Update API bindings:

- `getDestinationSettings`
- `chooseRootMediaFolder`
- `clearRootMediaFolder`
- `setForceProcessedFolder`
- planning/execution request changes if commands require explicit settings context

## Destination Root Rules

Effective destination root resolution:

- If `force_processed_folder` is true, use `<selected folder>/Processed`.
- Else if `root_media_folder` is set, use that folder after existence/readability validation.
- Else use `<selected folder>/Processed`.

The planner appends the existing year/month/day path below the effective destination root. Existing filename and conflict logic stays unchanged.

The mover validates:

- Sources are not inside the effective destination root.
- Destinations are inside the effective destination root.
- No destination overwrite occurs.
- Parent directories are created only under the effective destination root.

## Compatibility

Existing behavior remains the default because a fresh install has no custom root Media folder and `force_processed_folder` is false.

History files created before this feature will not contain destination root fields. Loading must tolerate missing fields. Undo/redo continues to operate from recorded source and destination paths.

## Testing Strategy

Rust unit tests:

- settings default load when no file exists
- settings persist and reload
- invalid settings JSON falls back or returns a clear error safely
- effective destination root resolves to custom root when configured
- effective destination root resolves to `<selected>/Processed` when forced
- planner builds date folders under custom root
- mover accepts custom-root destinations
- mover rejects destinations outside the effective root
- mover rejects sources inside the effective root
- history accepts missing destination root fields for old records

Frontend tests:

- settings menu renders configured root and force toggle
- choosing a root refreshes visible settings
- clearing a root refreshes visible settings
- force toggle updates state and leaves root path visible
- planning/execution controls show custom-root validation errors

## First Milestone

1. Add persisted destination settings commands.
2. Add shared effective destination root resolution.
3. Update planner and mover to use the effective destination root.
4. Add frontend right-side settings menu and API bindings.
5. Record destination root mode/path in new history runs.
6. Validate with Rust tests and frontend build/tests.

## Risks

- A plan could be generated under one root and executed after settings changed. Mitigation: include effective destination root in the plan/execution request or revalidate all operation destinations against the exact root used for that run.
- Custom roots can overlap source folders. Mitigation: reject processing sources inside the effective destination root and surface the validation result before moves.
- AppData settings corruption can break startup. Mitigation: use versioned defaults and clear error reporting.
