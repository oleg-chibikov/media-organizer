# Feature Specification: Processing History With Undo And Redo

**Feature Branch**: `003-history`  
**Created**: 2026-05-24  
**Status**: Draft  
**Input**: User wants each processing run to store logs in dedicated files under the program's appData folder, and the program should provide a menu that can undo or redo processing based on those log files.

## User Scenarios & Testing

### Primary User Story

As a user who processes media folders over time, I want the app to keep a durable processing history in the program's appData directory so I can inspect previous runs and use menu actions to undo or redo a completed processing run based on its recorded file operations.

### Acceptance Scenarios

1. **Run history file creation**
   - Given the user confirms processing for one or more files
   - When the processing run starts
   - Then the app creates a dedicated history log file for that run under the app's appData folder
   - And the log records enough information to undo and redo every attempted file operation

2. **History survives folder changes**
   - Given the user processed files from one TODO folder
   - When the user later opens the app without that folder selected
   - Then the history menu can still list the previous processing run from appData

3. **Undo processing**
   - Given a completed processing run has at least one undoable successful move
   - When the user chooses Undo for that run from the history menu
   - Then the app moves each destination file back to its recorded source path when safe
   - And the app records the undo result in the same run history

4. **Redo processing**
   - Given a processing run has been undone successfully
   - When the user chooses Redo for that run from the history menu
   - Then the app reapplies the recorded destination moves when safe
   - And the app records the redo result in the same run history

5. **Safety before undo or redo**
   - Given an undo or redo would overwrite an existing file or the expected source/destination is missing
   - When the user invokes the menu action
   - Then the app blocks or skips the unsafe file operation
   - And the app shows a clear result for the affected file

### Edge Cases

- The appData history directory does not exist.
- A history log file is partially written because the app crashed.
- A previous run contains failed, skipped, or copy-keep-source results.
- A destination file was moved, deleted, or modified outside the app after processing.
- A source path already exists when undo is requested.
- A destination path already exists when redo is requested.
- A run spans multiple selected folders or storage volumes.
- A NAS folder is unavailable when undo or redo is requested.
- The app cannot read or write appData because of permissions or profile corruption.
- Multiple app instances try to write history at the same time.
- The history menu contains many previous runs.
- A processing run used conflict suffixes, and redo must preserve the originally chosen destinations.

## Requirements

### Functional Requirements

- **FR-001**: The app shall store processing history under the program-specific appData directory provided by Tauri, not inside the selected TODO folder.
- **FR-002**: The app shall create one dedicated history log file per processing run.
- **FR-003**: The app shall name history files with a stable run id and timestamp so they are unique and sortable.
- **FR-004**: The app shall maintain a history index or discoverable manifest that allows the UI to list recent runs without scanning unrelated appData content.
- **FR-005**: Each run history shall record the run id, app version, selected folder, start timestamp, completion timestamp, final run status, and per-file operation records.
- **FR-006**: Each per-file operation record shall include original source path, planned destination path, final destination path, file size before move, file size after move when available, metadata date, original name, move strategy, status, error, and operation timestamp.
- **FR-007**: The app shall append undo and redo operation records to the same run history instead of rewriting the original processing records.
- **FR-008**: The app shall treat successful `rename` operations as undoable when the destination exists and the source path is free.
- **FR-009**: The app shall treat successful `copy_keep_source` operations as cleanup-undoable only when the original source still exists, the copied destination exists, the recorded sizes match, and deleting the copied destination will not remove the user's only known copy.
- **FR-010**: The app shall never overwrite an existing file during processing, undo, or redo.
- **FR-011**: The app shall verify expected file existence and size before each undo or redo file operation.
- **FR-012**: The app shall preserve the originally recorded destination path during redo, including conflict suffixes.
- **FR-013**: The app shall expose a History menu in the application menu bar.
- **FR-014**: The History menu shall list recent processing runs with timestamp, folder label, run status, and undo/redo availability.
- **FR-015**: The History menu shall provide Undo Processing and Redo Processing actions for the selected or most recent eligible run.
- **FR-016**: The app shall disable undo or redo menu actions when no eligible run exists.
- **FR-017**: The app shall show progress and per-file results while undo or redo is running.
- **FR-018**: The app shall allow the user to open a history details view for a run.
- **FR-019**: The app shall handle corrupt or partial history logs by showing them as incomplete and preventing unsafe undo/redo actions.
- **FR-020**: The app shall keep the existing TODO-folder processing log only as an optional compatibility/export artifact; appData history is the source of truth for undo/redo.

### Non-Functional Requirements

- **NFR-001**: History writes shall be append-only and flushed frequently enough to survive crashes with minimal data loss.
- **NFR-002**: Undo and redo shall use the same file safety rules as normal move execution.
- **NFR-003**: Reading history shall not block app startup; the UI may load history asynchronously.
- **NFR-004**: The history format shall be versioned so future app versions can migrate or ignore incompatible records safely.
- **NFR-005**: The history menu shall remain usable with hundreds of run logs by showing a bounded recent list and moving full browsing to the details view.

## Data Model

### HistoryRun

- `run_id`: Stable unique identifier.
- `history_version`: Version of the history file format.
- `app_version`: App version that created the run.
- `selected_folder`: Absolute selected folder path at processing time.
- `folder_label`: Display label for menus.
- `started_at`: Run start timestamp.
- `completed_at`: Optional run completion timestamp.
- `run_status`: `running`, `completed`, `completed_with_errors`, `undone`, `partially_undone`, `redone`, `partial`, or `corrupt`.
- `history_file_path`: Absolute appData path to the run log.

### HistoryOperation

- `operation_id`: Stable unique identifier.
- `run_id`: Associated run id.
- `file_id`: Original file id from processing.
- `operation_kind`: `process`, `undo`, or `redo`.
- `source_path`: Source path used by this operation.
- `destination_path`: Destination path used by this operation.
- `expected_source_size`: Optional expected source size before operation.
- `expected_destination_size`: Optional expected destination size after operation.
- `metadata_date`: Optional metadata date used for planning.
- `original_name`: Parsed original name.
- `move_strategy`: `rename`, `copy_keep_source`, or `not_attempted`.
- `status`: `succeeded`, `skipped`, `blocked`, or `failed`.
- `error`: Optional error message.
- `timestamp`: Operation timestamp.

### HistoryIndexEntry

- `run_id`: Run id.
- `folder_label`: Display label.
- `selected_folder`: Absolute selected folder path.
- `started_at`: Run start timestamp.
- `completed_at`: Optional completion timestamp.
- `run_status`: Current run status.
- `undo_available`: Whether at least one safe undo candidate exists.
- `redo_available`: Whether at least one safe redo candidate exists.
- `history_file_path`: Absolute appData path to the run log.

## Success Criteria

- **SC-001**: Every processing run creates a dedicated appData history log file.
- **SC-002**: Restarting the app still shows previous runs in the History menu.
- **SC-003**: The user can undo a completed run without overwriting files.
- **SC-004**: The user can redo an undone run using the original recorded destination paths.
- **SC-005**: Corrupt, incomplete, or unsafe history entries do not allow destructive undo/redo.
- **SC-006**: Undo and redo results are appended to the run history and visible in the details view.

## Out Of Scope For First Milestone

- Cross-device rollback with cryptographic hash verification.
- Editing or deleting history entries from inside the app.
- Syncing history across machines or user profiles.
- Undoing operations performed by older app versions without version-compatible history records.
- Global keyboard shortcuts for undo/redo.

## Resolved Decisions

- AppData history is the source of truth for undo/redo.
- Existing `Processed/processing-log.jsonl` logs are compatibility/export logs and are not sufficient for undo/redo.
- Redo uses the original recorded destination path rather than recalculating a new destination.
- Undo of `copy_keep_source` means removing the copied destination only after validation proves the original source is still present.
- Unsafe undo/redo operations are skipped or blocked instead of forcing changes.
