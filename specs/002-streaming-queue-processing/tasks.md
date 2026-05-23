# Tasks: Streaming Queue Processing For Metadata And Moves

**Spec**: `specs/002-streaming-queue-processing/spec.md`  
**Plan**: `specs/002-streaming-queue-processing/plan.md`

## Implementation

- [x] T001 Refactor `src/App.tsx` planning trigger to compute destination plans incrementally as metadata arrives (remove global wait-for-all-metadata gate).
- [x] T002 Add frontend move-processing queue orchestration in `src/App.tsx` so confirm starts dispatch immediately when ready plans exist and continues consuming newly-ready plans.
- [ ] T003 Add pause/resume controls and state in `src/App.tsx` and execution UI to stop/start dispatching new move jobs without losing queue contents.
- [ ] T004 Update API/type usage and run validation checks via `mise` commands; fix any compile/type issues caused by the queue refactor.
