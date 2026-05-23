# Implementation Plan: Streaming Queue Processing For Metadata And Moves

**Spec**: `specs/002-streaming-queue-processing/spec.md`  
**Stack**: Tauri 2, React, TypeScript, Rust  
**Milestone Target**: Incremental plan generation + queue-driven move execution with pause/resume

## Architecture Changes

The queue orchestration will be frontend-driven in `src/App.tsx`, reusing existing backend commands.

1. Metadata queue continues to read metadata per file.
2. Planning is triggered incrementally from current metadata/file snapshots rather than waiting for all media rows.
3. Move-ready plans feed a move queue.
4. Confirm/execute toggles execution mode; dispatcher starts immediately when queue has items.
5. Pause/resume controls gate dispatch of new move jobs.

## Backend Impact

No high-risk backend rewrite is required for this milestone.

- Keep `create_move_plan` and `execute_move_plan` commands.
- Reuse move events to capture per-file results.
- Execute small batches from frontend (single file or bounded chunk) to preserve pause responsiveness.

## Frontend State/Flow

- Add execution state:
  - `executionRequested`
  - `isMovePaused`
  - queue refs for queued/in-flight/completed move ids
- Replace "plan only after all metadata done" effect with incremental planner trigger.
- Add move dispatcher effect:
  - runs only when `executionRequested && !isMovePaused`
  - dispatches from queued ready ids
  - marks in-flight to prevent duplication
- Add pause/resume button in execution section.

## Validation

- `mise x -- pnpm test` (if configured) or targeted checks.
- `mise x -- cargo test` for Rust regressions.
- `mise x -- pnpm build` for frontend type/build validation.
