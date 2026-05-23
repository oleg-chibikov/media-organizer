# Agent Execution Rules

Use `mise` to run all project tools and scripts. Do not call `cargo`, `node`, `pnpm`, `npm`, or `tauri` directly.

## Required Pattern

- Use: `mise x -- <command> ...`

## Examples

- Rust tests: `mise x -- cargo test`
- Frontend build: `mise x -- pnpm build`
- Tauri dev: `mise x -- pnpm tauri dev`
- Install JS deps: `mise x -- pnpm install`
