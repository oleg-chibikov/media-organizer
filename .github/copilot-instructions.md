# Repository AI Instructions

This repository uses `AGENTS.md` as the primary agent contract.

Before coding:

1. Read `AGENTS.md`.
2. Read `docs/agent/quickstart.md`.
3. Read `docs/architecture.md` when changing behavior or command and event payloads.

Key project conventions:

- Use `mise x -- <command>` for project tooling.
- Keep filesystem logic in Rust (`src-tauri/src`), not React.
- Tauri payload types in `src/lib/types/generated/` come from `src-tauri/src/models/`. Regenerate them with `mise x -- cargo test --manifest-path src-tauri/Cargo.toml export_bindings`.
- Follow the layering in `docs/architecture.md`: Tauri types stay in `commands/` and `events.rs`, features import each other through `index.ts`.
- Favor targeted validation before broader checks.
