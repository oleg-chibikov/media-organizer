# Agent Execution Rules

Use `mise` to run all project tools and scripts. Do not call `cargo`, `node`, `pnpm`, `npm`, or `tauri` directly.
ON every change mark the corresponding task in the tasks.md as completed if relevant.

## Git Requirement

Agents must commit on every change.

## Required Pattern

- Use: `mise x -- <command> ...`

## Examples

- Rust tests: `mise x -- cargo test`
- Frontend build: `mise x -- pnpm build`
- Tauri dev: `mise x -- pnpm tauri dev`
- Install JS deps: `mise x -- pnpm install`
