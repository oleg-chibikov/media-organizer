# Tasks: Custom Root Media Folder

**Spec**: `specs/004-custom-media-root/spec.md`  
**Plan**: `specs/004-custom-media-root/plan.md`

## Phase 1: Backend Settings

- [ ] T001 Add Rust destination settings models for persisted settings and effective destination root responses.
- [ ] T002 Add appData destination settings path resolution and default settings loading.
- [ ] T003 Add read/write persistence for `destination-settings.json`.
- [ ] T004 Add commands to get destination settings, choose/set/clear root Media folder, and set force `Processed`.
- [ ] T005 Register destination settings commands in Tauri.
- [ ] T006 Add Rust tests for default settings, persistence, clearing, and force toggle updates.

## Phase 2: Destination Root Resolution

- [ ] T007 Add a shared effective destination root resolver for a selected TODO folder plus destination settings.
- [ ] T008 Validate configured root Media folder existence, directory type, and readability.
- [ ] T009 Preserve legacy `<selected folder>/Processed` behavior when no custom root is set.
- [ ] T010 Force `<selected folder>/Processed` when the force toggle is enabled even if a custom root is set.
- [ ] T011 Add Rust tests for custom root, legacy fallback, forced legacy, and invalid custom root resolution.

## Phase 3: Planning Integration

- [ ] T012 Update move planning commands to resolve and use the effective destination root.
- [ ] T013 Update `planner.rs` to build date-based destination directories under an injected destination root instead of hard-coded `Processed`.
- [ ] T014 Preserve existing filename generation, conflict suffixes, metadata date behavior, and streaming plan events.
- [ ] T015 Surface custom-root validation failures as planning errors without falling back silently.
- [ ] T016 Add Rust tests for planning under custom root and forced `Processed`.

## Phase 4: Move Safety And Execution

- [ ] T017 Update move execution commands to resolve or receive the same effective destination root used by planning.
- [ ] T018 Update `mover.rs` safety validation to allow destinations inside the effective destination root.
- [ ] T019 Update `mover.rs` safety validation to reject sources inside the effective destination root.
- [ ] T020 Keep no-overwrite and parent-directory safety behavior unchanged.
- [ ] T021 Add Rust tests for custom-root move validation, outside-root rejection, and source-inside-root rejection.

## Phase 5: History Compatibility

- [ ] T022 Add destination root mode/path fields to new history runs.
- [ ] T023 Record the effective destination root used by each processing run.
- [ ] T024 Keep history loading compatible with existing history files that do not include destination root fields.
- [ ] T025 Add Rust tests for history compatibility and new destination root fields.

## Phase 6: Frontend Settings Menu

- [ ] T026 Add TypeScript types and API helpers for destination settings commands.
- [ ] T027 Load destination settings on app startup.
- [ ] T028 Add a right-side settings menu showing root Media folder status and validation errors.
- [ ] T029 Add choose root, clear root, and force `Processed` controls.
- [ ] T030 Disable or defer settings changes while move execution is active.
- [ ] T031 Ensure planning and execution use current destination settings behavior.
- [ ] T032 Add frontend tests for settings menu rendering and interactions.

## Phase 7: Validation

- [ ] T033 Run `mise x -- cargo test`.
- [ ] T034 Run `mise x -- pnpm test`.
- [ ] T035 Run `mise x -- pnpm build`.
- [ ] T036 Smoke-test custom root mode and forced `Processed` mode on a temporary folder.
