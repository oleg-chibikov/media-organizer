# Feature Specification: Organize NAS Media TODO Folder

**Feature Branch**: `001-organize-nas-media`  
**Created**: 2026-05-23  
**Status**: Draft  
**Input**: User wants a fast Windows desktop GUI app that organizes a NAS TODO folder of photos/videos by metadata capture date, previews all changes lazily, and only moves files after confirmation.

## User Scenarios & Testing

### Primary User Story

As a user with a NAS folder containing unorganized photos and videos, I want to select the TODO folder, see a lazy preview of where each media file will be moved and how it will be renamed based on actual metadata capture date, and then confirm the operation so the app safely moves files into a `Processed/YYYY/MM/DD` structure.

### Acceptance Scenarios

1. **Folder selection**
   - Given the app is open
   - When the user selects a TODO folder
   - Then the app validates that the folder exists and is readable
   - And the app starts scanning files recursively

2. **Recursive discovery**
   - Given a selected TODO folder with files at multiple nesting levels
   - When scanning begins
   - Then candidate media files are discovered recursively
   - And files already under `Processed/` are skipped
   - And discovered files appear in the UI as they are found

3. **Original filename parsing with parentheses**
   - Given a file named `2024-11-02 13-09-41 (PXL_20241102_020933409.LS).mp4`
   - When the app parses the original filename
   - Then the original name is `PXL_20241102_020933409.LS`
   - And the final extension remains `.mp4`

4. **Original filename parsing without parentheses**
   - Given a file named `PXL_20260305_072017616.LS.mp4`
   - When the app parses the original filename
   - Then the original name is `PXL_20260305_072017616.LS`
   - And the final extension remains `.mp4`

5. **Metadata-driven date**
   - Given a media file whose filename contains a date
   - And whose metadata capture date is different
   - When the app creates the planned rename
   - Then the new filename uses the metadata capture date
   - And the filename date is ignored

6. **Lazy metadata preview**
   - Given a folder with many media files
   - When scanning and metadata extraction are running
   - Then the UI remains responsive
   - And each row updates when that file's metadata result is available

7. **Dry-run before move**
   - Given metadata has been extracted for at least one file
   - When the app computes the move plan
   - Then the UI shows source path, destination path, chosen metadata date, original name, and status
   - And no file is moved before the user confirms

8. **Safe execution**
   - Given the user confirms the move plan
   - When the app moves files
   - Then no destination file is overwritten
   - And cross-device NAS moves fall back from rename to copy-verify-remove
   - And the app updates each row with success or failure

9. **Processing log**
   - Given one or more files were processed
   - When the operation completes
   - Then the app writes a JSONL log under `TODO/Processed/processing-log.jsonl`
   - And each processed file has a durable source, destination, date, status, and error record

### Edge Cases

- A file has no usable metadata date.
- A metadata date has no timezone.
- A video metadata date is timezone-aware or UTC.
- Two files resolve to the same destination filename.
- The destination file already exists before processing starts.
- The destination file appears between dry-run and execution.
- A source file disappears between scan and execution.
- A NAS rename fails because the source and destination are on different filesystems/devices.
- The app loses access to the NAS during scan, metadata extraction, or move.
- ExifTool crashes or stops responding while the metadata queue is active.
- A selected folder is already inside `Processed/`.
- Non-media files or sidecar files are present in the TODO folder.
- Filenames include multiple parenthesized sections.
- Filenames include characters invalid for Windows destination paths.

## Requirements

### Functional Requirements

- **FR-001**: The app shall provide a native folder picker for selecting a TODO folder.
- **FR-002**: The app shall recursively scan the selected folder.
- **FR-003**: The app shall skip files under any `Processed/` folder within the selected TODO folder.
- **FR-004**: The app shall identify supported media files by extension, including `jpg`, `jpeg`, `png`, `heic`, `heif`, `tif`, `tiff`, `webp`, `mp4`, `mov`, `m4v`, `avi`, `mts`, and `m2ts`.
- **FR-005**: The app shall show unsupported and non-media files as red skipped/unhandled rows and shall never move them.
- **FR-006**: The app shall parse the original filename from the content inside parentheses when a filename follows the existing renamed format.
- **FR-007**: The app shall use the full filename stem as the original name when no parenthesized original name is detected.
- **FR-008**: The app shall read photo and video metadata using a bundled ExifTool binary, not a system-wide ExifTool dependency.
- **FR-009**: The app shall choose photo dates using this priority: `SubSecDateTimeOriginal`, `DateTimeOriginal`, `CreateDate`, `ModifyDate`, then `FileModifyDate` as a warning fallback when no EXIF capture date is available.
- **FR-010**: The app shall choose video dates using this priority: `MediaCreateDate`, `TrackCreateDate`, `CreateDate`, `ModifyDate`, then `FileModifyDate` as a warning fallback when no video capture date is available.
- **FR-011**: The app shall never use a filename date as the rename date.
- **FR-012**: The app shall use the user's local timezone for all preview and filename generation.
- **FR-013**: The app shall treat timezone-less metadata as local wall time and mark it as timezone unknown in the preview.
- **FR-028**: The app shall mark files using `FileModifyDate` fallback with a warning status in the preview.
- **FR-014**: The app shall compute destination directories as `SelectedTODOFolder/Processed/YYYY/MM/DD`.
- **FR-015**: The app shall compute destination filenames as `YYYY-MM-DD HH-MM-SS (original_name).ext`.
- **FR-016**: The app shall avoid overwrites by using suffixes in the format ` - 2`, ` - 3`, and so on before the extension.
- **FR-017**: The app shall detect conflicts against existing files and against other planned operations in the same run.
- **FR-018**: The app shall recheck destination conflicts immediately before each move.
- **FR-019**: The app shall require explicit user confirmation before moving or renaming files.
- **FR-020**: The app shall move files with atomic rename where possible.
- **FR-021**: The app shall fall back to copy-verify-remove when rename fails because of cross-device or NAS filesystem behavior.
- **FR-022**: The app shall not delete a source file unless the destination copy succeeded and basic verification passed.
- **FR-023**: The app shall update the UI lazily as scan, metadata, planning, and move statuses change.
- **FR-024**: The app shall keep the UI responsive for large folders by virtualizing the file list.
- **FR-025**: The app shall allow scan cancellation.
- **FR-026**: The app shall recover from an ExifTool process crash by failing the active request clearly, restarting when safe, and continuing or surfacing a recoverable error.
- **FR-027**: The app shall write a processing log in JSONL format under `SelectedTODOFolder/Processed/processing-log.jsonl`.

### Non-Functional Requirements

- **NFR-001**: The app shall be built as a Windows desktop app using Tauri 2, React, TypeScript, Vite, Tailwind CSS, and Rust.
- **NFR-002**: The final distribution shall not require users to install Rust, Node.js, pnpm, Python, Git, or ExifTool.
- **NFR-003**: The installer shall handle WebView2 availability for supported Windows systems.
- **NFR-004**: Metadata extraction shall avoid spawning a new ExifTool process per file once the persistent metadata worker is implemented.
- **NFR-005**: NAS load shall be controlled with bounded metadata concurrency.
- **NFR-006**: File operations shall be crash-conscious and shall prefer preserving source files over completing partial moves.
- **NFR-007**: The preview shall be the source of truth for what the app will attempt to move.

## Data Model

### FileRecord

- `id`: Stable row identifier.
- `source_path`: Absolute source file path.
- `relative_path`: Path relative to the selected TODO folder.
- `extension`: Original file extension.
- `kind`: `photo`, `video`, `unsupported`, or `unknown`.
- `scan_status`: `discovered`, `skipped_processed`, `skipped_unsupported`, or `scan_error`.

### MetadataResult

- `file_id`: Associated file record.
- `chosen_date`: Parsed date used for planning, if available.
- `chosen_date_source`: Metadata field used.
- `date_kind`: `timezone_aware`, `timezone_unknown`, or `fallback`.
- `raw_metadata_date`: Original metadata value.
- `metadata_status`: `pending`, `ready`, `no_date`, `error`, or `exiftool_restarted`.
- `error`: Optional error message.

### MovePlan

- `file_id`: Associated file record.
- `original_name`: Name derived from filename parsing.
- `destination_dir`: Planned `Processed/YYYY/MM/DD` directory.
- `destination_filename`: Planned final filename.
- `destination_path`: Full destination path.
- `conflict_status`: `none`, `suffix_applied`, or `blocked`.
- `plan_status`: `ready`, `blocked_no_date`, `blocked_conflict`, or `error`.

### MoveResult

- `file_id`: Associated file record.
- `attempted_source_path`: Source used at execution time.
- `attempted_destination_path`: Destination used at execution time.
- `move_strategy`: `rename`, `copy_remove`, or `not_attempted`.
- `result_status`: `moved`, `skipped`, or `failed`.
- `error`: Optional error message.

## Success Criteria

- **SC-001**: A user can select a TODO folder and see discovered files appear before the full scan completes.
- **SC-002**: For files with metadata dates, the app previews the exact final `Processed/YYYY/MM/DD` destination and filename.
- **SC-003**: Existing filename dates never affect the chosen rename date.
- **SC-004**: The app never overwrites an existing file during dry-run planning or execution.
- **SC-005**: The UI remains responsive while scanning and reading metadata from a large NAS folder.
- **SC-006**: Cross-device/NAS move failures are handled without data loss.
- **SC-007**: The app can produce a useful dry-run preview without moving any files.
- **SC-008**: The user can inspect skipped/unhandled files and metadata failures before confirming.
- **SC-009**: The app writes a JSONL log for completed move attempts.

## Out of Scope For First Milestone

- Moving `.aae`, `.xmp`, `.json`, or other sidecar files together with media.
- Duplicate detection by file hash.
- Undo operation.
- Cloud sync integration.
- macOS/Linux packaging.
- Portable single-file `.exe` distribution.
- Editing metadata.

## Resolved Decisions

- RAW formats are out of scope for the first release.
- `FileModifyDate` is used automatically only when no stronger metadata capture date is available, and the row is marked as a warning.
- The app uses the user's local timezone only. There is no explicit timezone selection in the first release.
- Unsupported files are listed as red skipped/unhandled rows and are never moved.
