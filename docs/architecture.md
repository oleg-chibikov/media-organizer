# Architecture

## Overview

Media Organizer is a Tauri 2 desktop app. The React frontend owns presentation and queue orchestration; the Rust backend owns filesystem access, metadata extraction, move planning, move execution, settings, history, and safety checks.

## Main Flow

1. The user selects a TODO folder in the frontend.
2. `src/lib/api.ts` invokes Rust commands registered in `src-tauri/src/main.rs`.
3. `scanner.rs` recursively discovers files and emits `scan:*` events.
4. `App.tsx` queues supported media files for metadata reads.
5. `metadata.rs` reads capture metadata through persistent bundled ExifTool workers.
6. `planner.rs` creates destination previews under the effective destination root and emits `plan:*` events.
7. After confirmation, `mover.rs` validates and executes ready operations, emitting `move:*` events.
8. `history.rs` records each processing run in appData and supports undo/redo through recorded paths.

## Frontend

- `src/App.tsx`: app-level state machine for scanning, metadata batching, planning, move dispatch, pause/resume, history, and destination settings.
- `src/lib/api.ts`: typed Tauri command wrappers and event listener helpers.
- `src/lib/types.ts`: frontend copies of Rust payload shapes.
- `src/components/`: toolbar, status bar, file table, history details, and destination settings UI.
- `src/components/design-system/`: reusable low-level UI primitives.

The frontend should not perform filesystem decisions. It requests work, renders streamed events, and prevents duplicate queue dispatch by file id.

## Backend

- `main.rs`: Tauri setup, app state registration, command registration.
- `commands.rs`: command boundary, async task spawning, metadata queue limits, history command orchestration, explorer integration.
- `models.rs`: serializable payloads shared with the frontend.
- `events.rs`: event names and emit helpers.
- `scanner.rs`: recursive folder scanning, media classification, `Processed` skipping.
- `metadata.rs`: ExifTool sidecar resolution, worker lifecycle, metadata parsing, date selection.
- `planner.rs`: destination path generation, original-name parsing, conflict suffix allocation.
- `mover.rs`: move preparation, root safety validation, conflict recheck, rename/copy-verify execution.
- `history.rs`: appData JSONL run logs, history index, undo/redo candidate recovery.
- `settings.rs`: persisted destination settings and effective destination root resolution.
- `filename.rs`: original-name parsing and Windows filename sanitization.

## Data And Persistence

- Destination settings are appData JSON managed by `settings.rs`.
- Processing history is appData JSONL plus an index managed by `history.rs`.
- The bundled ExifTool sidecar is under `src-tauri/binaries/`.
- Feature intent and acceptance criteria live in `specs/`.

## Safety Invariants

- No file is moved before explicit user confirmation.
- Existing destination files are never overwritten.
- Planning conflicts are checked again immediately before execution.
- Sources inside the effective destination root are rejected to avoid reprocessing organized files.
- Cross-device/NAS rename failures fall back to verified copy while keeping the source.
- Undo/redo uses recorded operation paths and validates existence/size before acting.

## Tooling

Use `mise` for all project commands:

```powershell
mise x -- pnpm tauri dev
mise x -- pnpm test
mise x -- pnpm lint
mise x -- cargo test --manifest-path src-tauri/Cargo.toml
mise x -- pnpm build
```
