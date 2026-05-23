# Tasks: Organize NAS Media TODO Folder

**Spec**: `specs/001-organize-nas-media/spec.md`  
**Plan**: `specs/001-organize-nas-media/plan.md`  
**Approach**: Complete tasks in order. Each task should leave the project runnable or testable.

## Phase 0: Development Environment

- [ ] T001 Verify `mise` works in this repo with `mise install`.
- [ ] T002 Verify Rust target is `x86_64-pc-windows-msvc`.
- [ ] T003 Verify Microsoft C++ Build Tools are available.
- [ ] T004 Verify Node and pnpm are available through mise.

## Phase 1: Scaffold Runnable App

- [x] T005 Scaffold a Tauri 2 + React + TypeScript + Vite app in the repo.
- [x] T006 Keep existing `PLAN.md`, `mise.toml`, and `specs/` files intact during scaffold.
- [x] T007 Install and configure Tailwind CSS for the React/Vite frontend.
- [x] T008 Add base app shell with toolbar, status bar, and empty file table area styled with Tailwind utilities.
- [ ] T009 Verify the desktop app launches in dev mode.
- [ ] T010 Verify the frontend production build succeeds.
- [ ] T011 Verify the Rust backend compiles.

## Phase 2: Shared Types And IPC Skeleton

- [x] T012 Add Rust `models.rs` with `FileRecord`, `MetadataResult`, `MovePlan`, and `MoveResult` structs.
- [x] T013 Add TypeScript types matching the Rust IPC/event payloads.
- [x] T014 Add Rust `commands.rs` and `events.rs` modules.
- [x] T015 Add a simple health-check command from React to Rust.
- [ ] T016 Verify React can invoke Rust and render the health-check result.

## Phase 3: Folder Selection

- [x] T017 Add native folder picker command.
- [x] T018 Display selected folder path in the toolbar.
- [x] T019 Validate selected folder exists and is readable.
- [x] T020 Warn when selected folder is inside a `Processed` path.
- [x] T021 Add UI states for no folder, folder selected, and folder invalid.

## Phase 4: Recursive Scanner

- [x] T022 Add Rust `scanner.rs`.
- [x] T023 Add extension classification for supported photo/video formats, including `heic` and `heif`, with RAW formats out of scope for the first release.
- [x] T024 Add unsupported/non-media classification for `.txt`, `.json`, `.aae`, `.xmp`, and unknown extensions.
- [x] T025 Add `Processed/` skip detection.
- [x] T026 Add cancellation-aware scan loop using `CancellationToken`.
- [x] T027 Emit scan events incrementally for discovered, skipped, error, and finished states.
- [x] T028 Add React event listeners for scan events.
- [x] T029 Render streamed scan rows in the UI.
- [x] T030 Add progress counters for discovered, skipped, errors, and completed.
- [x] T031 Add a cancel scan button wired to Rust cancellation.
- [x] T032 Add Rust tests for extension classification and `Processed/` skip detection.

## Phase 5: Virtualized File Table

- [x] T033 Install and configure `@tanstack/react-virtual`.
- [x] T034 Replace the simple file list with a virtualized table.
- [x] T035 Add stable columns for status, source file, kind, original name, metadata date, destination, and issue.
- [ ] T036 Verify the table remains responsive with a generated large row set.

## Phase 6: Filename Parser

- [x] T037 Add Rust `filename.rs`.
- [x] T038 Parse parenthesized original names from renamed files.
- [x] T039 Parse full filename stems when no original-name parentheses exist.
- [ ] T040 Preserve original file extension.
- [x] T041 Sanitize destination filename components for Windows path safety.
- [x] T042 Add Rust tests for filenames with parentheses, no parentheses, multiple parentheses, and invalid characters.
- [ ] T043 Display parsed original name in the UI for scanned media files.

## Phase 7: ExifTool One-Shot Metadata

- [x] T044 Add `src-tauri/binaries/` directory.
- [x] T045 Add documentation for obtaining Windows ExifTool and renaming it to `exiftool-x86_64-pc-windows-msvc.exe`.
- [x] T046 Configure the ExifTool binary as a Tauri sidecar.
- [ ] T047 Add Rust `metadata.rs` one-shot ExifTool invocation.
- [ ] T048 Parse ExifTool JSON output into a typed metadata structure.
- [ ] T049 Read metadata for one selected media file.
- [ ] T050 Display raw detected metadata dates in the UI.
- [ ] T051 Surface a clear error when the sidecar binary is missing.

## Phase 8: Date Selection

- [ ] T052 Implement photo date priority selection.
- [ ] T053 Implement video date priority selection.
- [ ] T054 Use `FileModifyDate` as an automatic warning fallback only when no stronger capture date is available.
- [ ] T055 Parse timezone-aware timestamps.
- [ ] T056 Parse timezone-less timestamps as local wall time and mark timezone unknown.
- [ ] T057 Convert timezone-aware timestamps to local time for filename generation.
- [ ] T058 Add Rust tests for photo priority, video priority, fallback warning, timezone-aware, and timezone-less dates.
- [ ] T059 Display chosen date, date source, and date kind in the UI.

## Phase 9: Dry-Run Planner

- [ ] T060 Add Rust `planner.rs`.
- [ ] T061 Compute `Processed/YYYY/MM/DD` destination directories.
- [ ] T062 Compute `YYYY-MM-DD HH-MM-SS (original_name).ext` destination filenames.
- [ ] T063 Detect conflicts against existing destination files.
- [ ] T064 Detect conflicts against other planned operations in the same run.
- [ ] T065 Apply suffixes as ` - 2`, ` - 3`, and so on.
- [ ] T066 Mark rows with no stronger capture date as warning fallback rows using `FileModifyDate`.
- [ ] T067 Emit planned destination updates lazily to the UI.
- [ ] T068 Add Rust tests for destination planning and conflict suffix assignment.
- [ ] T069 Verify no file operations occur during dry-run planning.

## Phase 10: Persistent ExifTool Worker

- [ ] T070 Implement persistent `-stay_open` ExifTool worker after one-shot metadata is stable.
- [ ] T071 Add request/response framing.
- [ ] T072 Add per-request timeout.
- [ ] T073 Add worker process watchdog.
- [ ] T074 Add crash detection and restart behavior.
- [ ] T075 Add bounded metadata queue and configurable concurrency.
- [ ] T076 Verify metadata rows update lazily while the UI remains responsive.

## Phase 11: Safe Move Execution

- [ ] T077 Add Rust `mover.rs`.
- [ ] T078 Add explicit confirmation summary before execution.
- [ ] T079 Recheck destination conflicts immediately before moving.
- [ ] T080 Use atomic rename when possible.
- [ ] T081 Fall back to copy-verify-remove on cross-device/NAS rename failure.
- [ ] T082 Verify destination existence and size before deleting source.
- [ ] T083 Never overwrite existing files.
- [ ] T084 Emit move progress events.
- [ ] T085 Add Rust tests for conflict recheck and copy-verify-remove.

## Phase 12: Processing Log

- [ ] T086 Add Rust `logging.rs`.
- [ ] T087 Create `Processed/` when needed.
- [ ] T088 Write JSONL records to `Processed/processing-log.jsonl`.
- [ ] T089 Log source path, destination path, metadata date, original name, move strategy, status, error, and timestamp.
- [ ] T090 Verify failed and skipped moves are logged.

## Phase 13: Packaging

- [ ] T091 Add app icon and Windows metadata.
- [ ] T092 Configure Tauri NSIS installer.
- [ ] T093 Configure WebView2 bootstrapper handling.
- [ ] T094 Verify ExifTool sidecar is included in production build.
- [ ] T095 Build installer `.exe`.
- [ ] T096 Smoke-test installed app on a clean Windows user profile.

## Phase 14: Documentation

- [ ] T097 Add README with development setup using mise.
- [ ] T098 Add ExifTool sidecar setup instructions.
- [ ] T099 Add user workflow documentation.
- [ ] T100 Add safety notes about dry-run preview and copy-verify-remove behavior.

## First Useful Delivery

Complete T001 through T069 first.

This produces a useful dry-run app that selects a folder, scans files, parses original names, reads metadata with one-shot ExifTool, and previews destination paths without moving files.
