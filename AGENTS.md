# Agent Execution Rules

## Tooling

- Run all project tools through `mise`: `mise x -- <command> ...`.
- Do not call `cargo`, `node`, `pnpm`, `npm`, or `tauri` directly.
- Examples:
  - Rust tests: `mise x -- cargo test`
  - Frontend build: `mise x -- pnpm build`
  - Tauri dev app: `mise x -- pnpm tauri dev`
  - Install JS deps: `mise x -- pnpm install`

## Change Workflow

- Agents must commit every change.
- Check `git status --short` before editing and before committing.
- Do not revert user changes unless explicitly requested.
- If explicitly asked to complete a task from `specs/**/tasks.md`, mark the corresponding task complete in that file.
- Read `CONTRIBUTING.md` when making code changes or validating project workflow.

## Repo Context

- Frontend: React + TypeScript + Vite in `src/`.
- Backend: Tauri 2 + Rust in `src-tauri/src/`.
- Runtime bridge: Tauri commands/events are wrapped by `src/lib/api.ts`.
- Source of truth for app behavior is the Rust backend; frontend state should mirror command results and emitted events.
- Architecture notes live in `docs/architecture.md`.

## Agent Context Files

- Start with `docs/agent/quickstart.md` for token-efficient project conventions.
- Read additional files in `docs/agent/upstream/` only when needed for the current task.
- Refresh vendored upstream docs with `powershell -File scripts/sync-agent-docs.ps1`.

## Verification

- Prefer targeted checks first, then broader checks when practical:
  - `mise x -- pnpm test`
  - `mise x -- pnpm lint`
  - `mise x -- cargo test --manifest-path src-tauri/Cargo.toml`
  - `mise x -- pnpm build`
