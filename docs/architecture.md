# Architecture

Media Organizer is a Tauri 2 desktop app. The Rust backend owns every filesystem decision: scanning, metadata, planning, moves, history, settings and safety checks. The React frontend renders backend state, queues work and sends confirmed batches.

## Main Flow

1. The user picks a TODO folder. `features/scan` calls `start_scan`.
2. `scanner` walks the folder on the blocking pool and streams `scan:*` events.
3. `features/metadata` batches media rows into `read_metadata_for_files`. `metadata_queue` reads them through persistent ExifTool workers.
4. `features/planning` sends rows with metadata to `create_move_plan`. `planner` streams `plan:*` events with destination previews.
5. After the user confirms, `features/moves` sends ready plans to `execute_move_plan`. `move_run` checks, moves and records each file and streams `move:*` events.
6. `history` keeps a JSONL log per run in appData. `undo_redo` replays a run in either direction and streams `history:*` events.

## Layering

Each layer may depend only on the layers below it.

| Layer         | Backend (`src-tauri/src`)                                               | Frontend (`src`)                                      |
| ------------- | ----------------------------------------------------------------------- | ----------------------------------------------------- |
| Entry         | `main.rs` registers state and commands                                  | `main.tsx`, `App.tsx` lays out features               |
| Orchestration | `commands/` thin async handlers                                         | `app/` wires features together                        |
| Features      | domain modules listed below                                             | `features/<name>/`, used only through `index.ts`      |
| Shared        | `models/`, `events.rs`, `path_safety.rs`, `file_ops/`, `atomic_file.rs` | `lib/api/`, `lib/types/`, `components/design-system/` |

- **Tauri stays at the edge.** Only `main.rs`, `commands/` and `events.rs` use `AppHandle`. Domain modules take paths, stores and an `EventSink`.
- **Commands stay thin.** A handler resolves app paths in `commands/app_dirs.rs`, runs filesystem work through `commands/background.rs`, and delegates. Validation and orchestration live in the domain module, as in `move_run::start_move_run` and `undo_redo::prepare_replay`.
- **No blocking on the UI thread.** Commands that touch the filesystem are `async`. Long jobs use `spawn_job` and report through events. Folder pickers use the callback dialog.
- **Features talk through indexes.** ESLint rejects deep imports into another feature, feature imports of `app/`, and `lib/` or design-system imports of features.
- **Errors cross the boundary as `String`.** Internal enums such as `WorkerError` and `OperationOutcome` exist only where code branches on the error kind.

## Backend Modules

- `commands/`: one file per command group, plus `app_dirs`, `background` and `folder_dialog` helpers.
- `models/`: serde payloads shared with `src/lib/types/`, split by domain and re-exported as `crate::models::*`.
- `events.rs`: event names, the `EventSink` trait and typed emit helpers. Tests use `test_support::events::RecordingSink`.
- `scanner/`: recursive walk, media classification, `Processed` skipping and the `ActiveScan` cancel handle.
- `metadata/`: ExifTool sidecar lookup, worker lifecycle, JSON parsing and capture-date selection.
- `metadata_queue/`: bounded batch queue over the worker pool, with env-configurable limits.
- `planner/`: dated destination paths, original-name filenames and conflict suffixes. Planning is a dry run.
- `mover/`: move preparation, root safety checks, conflict recheck, rename with verified copy fallback, and the confirmation summary.
- `move_run/`: one confirmed batch from start to finish: history run, per-file progress, results and source folder cleanup.
- `history/`: `HistoryStore` over run logs and the bounded index, crash recovery and undo/redo candidates.
- `undo_redo/`: `ReplayDirection` eligibility, per-operation checks and the `ReplayJob` that logs each step.
- `settings/`: `SettingsStore` and effective destination root resolution.
- `filename/`, `system_explorer/`, `path_safety.rs`, `file_ops/`, `atomic_file.rs`: focused helpers named after what they decide.

## Frontend Modules

- `app/useAppSession.ts`: composes the scan, metadata, planning, moves, history and destination hooks.
- `app/useBackendBootstrap.ts`: loads health, processing limits and settings at startup.
- `app/useDestinationChange.ts`: applies a destination change and replans before any move starts.
- `features/`: `scan`, `metadata`, `planning`, `moves`, `history`, `file-table`, `status-bar`, `destination-settings`, `toolbar`.
- `lib/api/`: typed command wrappers, `listenAll` event subscriptions and `useBackendEvents`.
- `lib/types/`: TypeScript copies of the Rust payloads.

The frontend doesn't decide filesystem outcomes. It renders streamed events, derives tallies from them and prevents duplicate dispatch by file id.

## Data And Persistence

- **Settings** are `<appData>/settings/destination-settings.json`, written by `SettingsStore`.
- **History** is `<appData>/history/runs/*.jsonl` plus `index.json`, written by `HistoryStore`.
- **Writes** to settings and the history index take a process lock and replace the file through a temp file.
- **ExifTool** ships as a sidecar under `src-tauri/binaries/`.
- **Specs** with feature intent and acceptance criteria live in `specs/`.

## Safety Invariants

- No file moves before the user confirms.
- Existing destination files are not overwritten. Copies publish with `create_new`.
- Conflicts are checked again right before each move.
- Sources must sit inside the selected folder. Destinations must sit inside the effective destination root.
- Cross-device rename failures fall back to a verified copy that keeps the source.
- Undo and redo use recorded paths and sizes. A replay stops when its log can't record a step.
