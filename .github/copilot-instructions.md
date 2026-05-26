# Repository AI Instructions

This repository uses `AGENTS.md` as the primary agent contract.

Before coding:

1. Read `AGENTS.md`.
2. Read `docs/agent/quickstart.md`.
3. Read deeper references from `docs/agent/upstream/` only if needed for the task.

Key project conventions:

- Use `mise x -- <command>` for project tooling.
- Keep filesystem logic in Rust (`src-tauri/src`), not React.
- Keep Tauri command payloads aligned between Rust models and `src/lib/types.ts`.
- Favor targeted validation before broader checks.
