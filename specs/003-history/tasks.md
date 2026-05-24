# Tasks: Processing History With Undo And Redo

**Spec**: `specs/003-history/spec.md`  
**Plan**: `specs/003-history/plan.md`

## Phase 1: History Storage

- [x] T001 Add Rust history models for `HistoryRun`, `HistoryOperation`, and `HistoryIndexEntry`.
- [x] T002 Add appData history path resolution using Tauri's app data directory.
- [x] T003 Create `appData/history/runs/` when processing starts.
- [x] T004 Generate stable sortable run log filenames with timestamp and run id.
- [x] T005 Write run start and run completion records to append-only JSONL.
- [x] T006 Maintain `appData/history/index.json` with bounded recent-run summaries.
- [x] T007 Add Rust tests for path construction, run filename generation, JSONL append, and index sorting.

## Phase 2: Processing Integration

- [x] T008 Start a history run when confirmed move processing begins.
- [x] T009 Append one processing operation record for every attempted move.
- [x] T010 Record original source path, final destination path, expected sizes, metadata date, original name, strategy, status, error, and timestamp.
- [x] T011 Mark runs as completed, completed with errors, partial, or corrupt based on processing and log state.
- [x] T012 Keep `Processed/processing-log.jsonl` as optional compatibility output while making appData history the undo/redo source of truth.

## Phase 3: History Loading

- [x] T013 Add `list_history_runs` command.
- [x] T014 Add `get_history_run` command.
- [x] T015 Recover run logs line by line and tolerate partial trailing records.
- [x] T016 Mark unreadable, malformed, or incomplete runs as unsafe for undo/redo.
- [x] T017 Add Rust tests for partial and corrupt history recovery.

## Phase 4: History Menu And Details

- [x] T018 Add a History menu to the app menu bar.
- [x] T019 List recent runs with timestamp, folder label, and status.
- [x] T020 Add disabled states for Undo Processing and Redo Processing when no eligible run exists.
- [x] T021 Add a history details view for per-file processing, undo, and redo records.
- [x] T022 Load history asynchronously after app startup and when history update events arrive.
- [x] T023 Add frontend tests for menu availability and details rendering.

## Phase 5: Undo

- [x] T024 Add undo eligibility calculation for completed run records, including different handling for `rename` and `copy_keep_source` results.
- [x] T025 Implement undo validation for destination exists, source path is free for `rename`, original source still exists for `copy_keep_source`, size matches when available, and no overwrite.
- [x] T026 Execute undo by moving recorded `rename` destinations back to recorded source paths and removing copied destinations only after `copy_keep_source` validation passes.
- [x] T027 Append undo operation results to the same run history log.
- [x] T028 Emit undo progress events.
- [x] T029 Add Rust tests for successful undo and blocked undo cases.

## Phase 6: Redo

- [ ] T030 Add redo eligibility calculation for undone run records.
- [ ] T031 Implement redo validation for source exists, destination path is free, size matches when available, and no overwrite.
- [ ] T032 Execute redo using the originally recorded destination paths without recalculating destinations.
- [ ] T033 Append redo operation results to the same run history log.
- [ ] T034 Emit redo progress events.
- [ ] T035 Add Rust tests for successful redo and blocked redo cases.

## Phase 7: Validation

- [ ] T036 Run `mise x -- cargo test`.
- [ ] T037 Run `mise x -- pnpm build`.
- [ ] T038 Smoke-test a process, undo, and redo workflow on a temporary folder.
