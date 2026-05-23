# Feature Specification: Streaming Queue Processing For Metadata And Moves

**Feature Branch**: `002-streaming-queue-processing`  
**Created**: 2026-05-23  
**Status**: Draft  
**Input**: User wants destination planning to happen immediately after each file's metadata is obtained, and move execution to start as soon as the user confirms and at least one movable item is available, without waiting for all files to finish metadata/planning. Processing must support pause/resume.

## User Scenarios & Testing

### Primary User Story

As a user processing a large TODO folder, I want metadata extraction and file moving to run as separate queues so destination previews appear incrementally and move execution can begin immediately after confirmation with currently-ready files, while still allowing me to pause move processing.

### Acceptance Scenarios

1. **Incremental destination planning**
   - Given scanning and metadata extraction are running
   - When metadata becomes ready for a file
   - Then destination planning for that file is triggered immediately
   - And the row destination/plan status updates without waiting for all metadata jobs

2. **Confirm starts queue-driven execution**
   - Given at least one file is in ready-to-move state
   - When the user presses confirm/execute
   - Then move execution starts immediately for queued ready files
   - And new ready files discovered later are also processed while execution remains active

3. **No global wait requirement**
   - Given metadata extraction is still ongoing for many files
   - When some files are already ready to move
   - Then execution does not block on unresolved files
   - And unresolved files remain pending until their metadata/plan becomes ready

4. **Pause and resume move processing**
   - Given move execution is active
   - When the user presses pause
   - Then no new move jobs are started
   - And when the user presses resume, queued jobs continue processing

## Requirements

### Functional Requirements

- **FR-001**: The app shall maintain a metadata extraction queue separate from a move processing queue.
- **FR-002**: The app shall trigger destination planning as soon as metadata for any media file becomes available.
- **FR-003**: The app shall not require all media files to have metadata before any move plan is shown.
- **FR-004**: The app shall allow user confirmation of execution once at least one move-ready plan exists.
- **FR-005**: After confirmation, the app shall start move processing immediately if the move queue is non-empty.
- **FR-006**: After confirmation, the app shall keep enqueueing newly-ready plans and continue processing them automatically.
- **FR-007**: The app shall provide a pause control for move processing that prevents dispatching new move jobs while preserving queued work.
- **FR-008**: The app shall provide a resume control that restarts dispatching queued move jobs.
- **FR-009**: Existing move safety rules shall remain unchanged (no overwrite, conflict recheck, rename/copy fallback).

### Non-Functional Requirements

- **NFR-001**: UI must remain responsive while scan, metadata, planning, and move processing run concurrently.
- **NFR-002**: Queue orchestration must avoid duplicate move execution for a single file id.
- **NFR-003**: Pause/resume behavior must be deterministic in frontend state (in-flight jobs may finish; no new jobs start while paused).

## Data Model Changes

- Introduce frontend-only execution control state:
  - `execution_requested`: whether user confirmed queue processing
  - `move_paused`: whether dispatching new jobs is paused
  - `queued_move_ids`: file ids ready and waiting for move dispatch
  - `inflight_move_ids`: file ids currently being executed
  - `completed_move_ids`: file ids already completed (moved/skipped/failed)

## Success Criteria

- **SC-001**: Destination path appears for each file shortly after its metadata result arrives.
- **SC-002**: User can start execution before scan/metadata completion and observe moves starting immediately.
- **SC-003**: Pausing execution stops new move dispatches and resuming continues from queued items.
- **SC-004**: No file is executed more than once in a single run.
