# Implementation Plan: Organize NAS Media TODO Folder

**Spec**: `specs/001-organize-nas-media/spec.md`  
**Stack**: Tauri 2, React, TypeScript, Vite, Tailwind CSS, Rust, bundled ExifTool  
**Milestone Target**: Dry-run preview through destination planning, no file moves

## Architecture

The application is a Windows desktop app with a React frontend and Rust backend.

React owns presentation state, folder selection flow, virtualized list rendering, progress counters, filters, and user confirmation.

Rust owns filesystem access, recursive scanning, filename parsing, metadata extraction, move planning, safe move execution, and JSONL logging.

Tauri IPC is used for commands that return immediate results. Tauri events are used for long-running streams such as scanning, metadata extraction, planning, and move progress.

## Project Structure

```text
src/
  App.tsx
  main.tsx
  components/
    FileTable.tsx
    Toolbar.tsx
    StatusBar.tsx
  lib/
    api.ts
    types.ts

src-tauri/
  src/
    main.rs
    commands.rs
    events.rs
    scanner.rs
    filename.rs
    metadata.rs
    planner.rs
    mover.rs
    logging.rs
    models.rs
  binaries/
    exiftool-x86_64-pc-windows-msvc.exe
```

## Frontend Design

The first screen is the working app, not a landing page.

Styling uses Tailwind CSS. Keep the visual system restrained and utility-focused: dense, readable tables; clear status colors; compact controls; and no marketing-style hero sections. Use component-level class composition where it improves readability, but avoid introducing a separate component library until the app's interaction model is stable.

Primary controls:

- Choose folder
- Start/cancel scan
- Metadata fallback toggle, initially disabled until metadata work exists
- Proceed button, disabled until ready move plans exist

Primary views:

- Progress counters
- Virtualized file table
- Details/error area for the selected row

Initial table columns:

- Status
- Source file
- Kind
- Original name
- Metadata date
- Destination
- Issue

Use `@tanstack/react-virtual` before the app is tested against large folders.

## Rust Modules

`models.rs` defines shared serializable types for frontend IPC and event payloads.

`scanner.rs` recursively scans the selected folder, classifies file kinds, skips `Processed/`, and emits file discovery events. It is cancellation-aware from the start.

`filename.rs` extracts original names and sanitizes destination-safe names.

`metadata.rs` wraps ExifTool. Start with a one-shot command to prove sidecar access, then implement persistent `-stay_open` workers with request framing, timeouts, watchdog, and restart behavior.

`planner.rs` combines file records and metadata results into dry-run move plans, including destination path creation and conflict suffix assignment.

`mover.rs` executes confirmed plans. It uses rename where possible and copy-verify-remove when rename fails because of cross-device/NAS behavior.

`logging.rs` writes JSONL operation records.

## IPC And Events

Commands:

- `choose_folder`
- `start_scan`
- `cancel_scan`
- `start_metadata`
- `cancel_metadata`
- `create_move_plan`
- `execute_move_plan`

Events:

- `scan:started`
- `scan:file`
- `scan:skipped`
- `scan:error`
- `scan:finished`
- `metadata:started`
- `metadata:file`
- `metadata:error`
- `metadata:finished`
- `plan:file`
- `plan:error`
- `move:file`
- `move:finished`

Long-running operations must emit incremental events instead of returning one huge payload.

## Date Handling

Photo priority:

1. `SubSecDateTimeOriginal`
2. `DateTimeOriginal`
3. `CreateDate`
4. `ModifyDate`
5. `FileModifyDate`, as a warning fallback when no stronger capture date is available

Video priority:

1. `MediaCreateDate`
2. `TrackCreateDate`
3. `CreateDate`
4. `ModifyDate`
5. `FileModifyDate`, as a warning fallback when no stronger capture date is available

Timezone-less metadata is treated as local wall time and shown as timezone unknown.

All preview and filename generation uses the user's local timezone. There is no explicit timezone selection in the first release.

Filename dates are never date sources.

Rows using `FileModifyDate` fallback must be visually marked as warnings.

Unsupported files are rendered as red skipped/unhandled rows and are never included in move plans.

## Destination Planning

Destination directory:

```text
SelectedTODOFolder/Processed/YYYY/MM/DD
```

Destination filename:

```text
YYYY-MM-DD HH-MM-SS (original_name).ext
```

Conflict suffix:

```text
YYYY-MM-DD HH-MM-SS (original_name) - 2.ext
YYYY-MM-DD HH-MM-SS (original_name) - 3.ext
```

Planner must check both existing destination files and duplicate destinations in the current run.

Executor must recheck destination conflicts immediately before moving.

## File Safety

Move execution is intentionally not part of the first milestone.

When implemented:

- Never overwrite.
- Prefer rename for same-filesystem moves.
- On cross-device/NAS rename failure, copy to destination.
- Verify destination size and existence.
- Delete source only after verification passes.
- Log every attempted operation.

## ExifTool Sidecar

Windows ExifTool is distributed as `exiftool(-k).exe`.

For Tauri sidecar use, the binary must be renamed with the target triple:

```text
src-tauri/binaries/exiftool-x86_64-pc-windows-msvc.exe
```

The one-shot wrapper must be implemented and tested before persistent mode.

Persistent mode with `-stay_open` is the highest-risk technical item. It requires:

- stdin/stdout request framing
- bounded queue
- per-request timeout
- process health tracking
- restart logic
- clear failure events

## Testing Strategy

Rust unit tests:

- extension classification
- `Processed/` skip detection
- filename parsing
- destination filename formatting
- conflict suffix assignment
- date priority selection
- copy-verify-remove behavior with temporary directories

Frontend tests:

- renders folder state
- appends streamed rows
- updates row status
- disables/enables action buttons

Acceptance tests can be added later as Gherkin/Cucumber-style feature files once core flows stabilize.

## First Milestone

The first milestone is a useful dry-run preview:

1. App launches.
2. User selects a folder.
3. Rust scans recursively and streams files to React.
4. UI shows discovered and skipped files.
5. Filename parser shows original names.
6. ExifTool one-shot metadata extraction works.
7. Planner previews destination path and final filename.
8. No files are moved.

## Risks

- `-stay_open` ExifTool IPC is technically tricky. Mitigation: prove one-shot first, then build persistent worker with tests and timeouts.
- NAS moves can be cross-device. Mitigation: explicit fallback to copy-verify-remove in executor.
- Timezone-less EXIF dates can be ambiguous. Mitigation: treat as local wall time and show a visible timezone unknown state.
- Large folders can overload the UI. Mitigation: stream events and virtualize rows from the start.
