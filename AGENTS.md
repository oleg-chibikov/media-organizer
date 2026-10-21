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

## Code Quality Bar

- Produce effective, performant, accessible, elegant, and very readable code.
- Prefer simple, explicit designs over clever abstractions; add abstractions only when they remove real duplication or clarify intent.
- Keep each file minimal and single-purpose. Split files when a second responsibility appears.
- Allocate feature work into feature folders when a change spans components, state, helpers, tests, or styles.
- Preserve semantic HTML, keyboard access, labels, focus behavior, and readable contrast for UI changes.
- Avoid unnecessary renders, repeated filesystem calls, unbounded queues, and work on the UI thread.
- Use descriptive names and direct control flow; comments should explain non-obvious reasoning, not restate code.
- Keep public command/event contracts typed and stable across Rust and TypeScript.

## Repo Context

- Frontend: React + TypeScript + Vite in `src/`.
- Backend: Tauri 2 + Rust in `src-tauri/src/`.
- Runtime bridge: Tauri commands and events are wrapped by `src/lib/api/`.
- Source of truth for app behavior is the Rust backend; frontend state should mirror command results and emitted events.
- Architecture notes live in `docs/architecture.md`.

## File Organization

- Put reusable low-level UI primitives in `src/components/design-system/`.
- Keep `src/App.tsx` to layout. Put state that crosses features in `src/app/`.
- Put frontend features in `src/features/<feature>/` and expose them only through its `index.ts`.
- Keep Tauri command wrappers and event subscriptions in `src/lib/api/`.
- Keep shared frontend payload types in `src/lib/types/` aligned with `src-tauri/src/models/`.
- Keep Tauri types in `main.rs`, `commands/` and `events.rs`. Domain modules take paths, stores and an `EventSink`.
- Keep command handlers thin and async; put validation and orchestration in the domain module.
- Put backend behavior in focused Rust modules; create a folder module when one backend feature needs multiple files.
- Keep files under about 100 lines, tests under about 150.

## Agent Context Files

- Start with `docs/agent/quickstart.md` for token-efficient project conventions.
- Read `docs/architecture.md` before changing command/event contracts, filesystem behavior, queueing, history, settings, or cross-boundary data flow.
- Use `docs/agent/README.md` to decide whether guidance belongs in `AGENTS.md`, repo docs, or a skill.

## Skills And Docs

- Keep always-on repository rules in `AGENTS.md`: tooling, workflow, invariants, and the quality bar.
- Keep deeper project explanations in `docs/agent/` or `docs/architecture.md`; docs should be concise and loaded only when relevant.
- Use repo skills in `.agents/skills/` only for repeatable task workflows with clear trigger descriptions.
- Do not put universal coding standards only in a skill, because skills are selected on demand.
- Keep skill instructions concise; reference docs instead of copying long guidance into `SKILL.md`.

## Verification

- Prefer targeted checks first, then broader checks when practical:
  - `mise x -- pnpm test`
  - `mise x -- pnpm lint`
  - `mise x -- cargo test --manifest-path src-tauri/Cargo.toml`
  - `mise x -- pnpm build`
