# Implementation Plan: Processing History With Undo And Redo

**Spec**: `specs/003-history/spec.md`  
**Stack**: Tauri 2, React, TypeScript, Rust  
**Milestone Target**: Durable appData run history with menu-driven undo/redo

## Architecture

History is owned by the Rust backend because it needs reliable appData path resolution, file locking, append-only writes, and safe filesystem operations.

React owns the History menu presentation, history details view, and progress display for undo/redo operations.

Tauri IPC commands load history summaries and start undo/redo operations. Tauri events stream history write status and undo/redo progress.

## AppData Layout

Use Tauri's app data directory for this app.

```text
appData/
  history/
    index.json
    runs/
      2026-05-24T10-15-30Z_<run_id>.jsonl
```

The run log is append-only JSONL. The index stores bounded summary data for menu loading and points to each run log.

## Backend Changes

Add or extend Rust modules:

- `history.rs`: appData path resolution, index read/write, run log creation, append-only history records, recovery from partial files.
- `undo.rs`: validates and executes undo operations from history records.
- `redo.rs`: validates and executes redo operations from history records.
- `commands.rs`: exposes history list/detail/undo/redo commands.
- `events.rs`: emits history and undo/redo progress events.
- `models.rs`: adds versioned history data types shared with the frontend.

The existing move executor should start a history run before processing begins and append a history operation for each attempted file operation. The existing TODO-folder JSONL processing log can remain for compatibility, but it should not be used as the undo/redo source.

## Frontend Changes

Add History menu support and a details surface:

- Load recent history summaries asynchronously after app startup.
- Show recent runs in the History menu with timestamp, folder label, and status.
- Enable Undo Processing or Redo Processing only when the selected or most recent run has eligible operations.
- Show a history details view with per-file process/undo/redo records.
- Display undo/redo progress using the same row/status patterns as normal processing.

## Undo Rules

Undo must validate each file before moving it back:

- Destination from the original process operation exists.
- Recorded source path does not already exist.
- Destination size matches the recorded expected size when available.
- Operation was a successful move strategy that is safe to reverse.
- No overwrite occurs.

For safe `rename` undo, move the recorded processing `destination_path` back to the recorded processing `source_path`. For validated `copy_keep_source` cleanup undo, remove only the recorded copied destination. If a file fails validation, append a blocked undo record and continue with the next eligible file.

## Redo Rules

Redo must validate each file before reapplying:

- Recorded source path exists.
- Recorded destination path does not already exist.
- Source size matches the expected original size when available.
- Original process operation is redoable.
- No destination recalculation occurs.

Redo uses the original recorded destination path, including conflict suffixes. If validation fails, append a blocked redo record and continue with the next eligible file.

## Crash Recovery

On startup, history loading should:

- Read `index.json`.
- Verify referenced run files exist.
- Parse JSONL line by line.
- Treat malformed trailing lines as partial records.
- Mark runs as `partial` or `corrupt` when necessary.
- Disable undo/redo for unsafe or unreadable runs.

## IPC And Events

Commands:

- `list_history_runs`
- `get_history_run`
- `undo_history_run`
- `redo_history_run`

Events:

- `history:index-updated`
- `history:run-started`
- `history:operation-written`
- `history:undo-progress`
- `history:redo-progress`
- `history:run-updated`

## Testing Strategy

Rust unit tests:

- appData history path construction
- run log file naming
- JSONL append and recovery from partial trailing lines
- index update and recent-run sorting
- undo validation blocks overwrite
- undo validation blocks missing destination
- redo validation preserves recorded destination
- corrupt history disables undo/redo

Frontend tests:

- History menu disables actions when no eligible run exists
- History menu lists recent runs loaded from backend
- Undo/redo progress updates are rendered
- History details view renders process, undo, and redo records

## First Milestone

The first milestone is durable history plus a safe undo path:

1. Create appData history directory and per-run log files.
2. Append processing operation records during move execution.
3. Maintain a recent-run index.
4. Add History menu entries for recent runs.
5. Implement safe undo for completed rename operations and validated cleanup undo for copied destinations.
6. Append undo results to the same run log.

Redo can follow once undo behavior and history recovery are proven.

## Risks

- Undo can be destructive if validation is weak. Mitigation: block on missing paths, existing targets, size mismatch, unreadable logs, or unsupported move strategies.
- AppData history can diverge from real filesystem state. Mitigation: validate every file immediately before undo/redo and record blocked operations.
- Partial logs can happen after crashes. Mitigation: append JSONL records, recover line by line, and disable unsafe actions for corrupt runs.
