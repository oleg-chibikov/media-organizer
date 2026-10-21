# Contributing

## Ground Rules

- Follow repository automation rules in `AGENTS.md`.
- Keep frontend command/event names in sync with Rust command handlers and event constants.
- Preserve filesystem safety rules: no overwrites, validate roots, and prefer preserving source files over completing risky moves.
- Update specs/tasks only when the work is explicitly tied to a spec task.

## Code Guidelines

- Follow the module map and layering rules in `docs/architecture.md`.
- Put cross-feature UI wiring in `src/app/` and feature state in `src/features/<feature>/`.
- Put Tauri command wrappers and event subscriptions in `src/lib/api/`.
- Keep shared frontend payload types in `src/lib/types/` aligned with Rust `src-tauri/src/models/`.
- Put filesystem behavior in Rust domain modules, not the frontend or the command handlers.
- Add or update tests around path safety, conflict handling, history, metadata date selection, and queue behavior when changing those areas.

## Validation

Run the smallest meaningful check for the change, then broader checks before committing when practical. Use the command wrapper defined in `AGENTS.md`.

## Commit Policy

Every change should be committed. Before committing:

```powershell
git status --short
```
