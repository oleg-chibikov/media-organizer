# Contributing

## Ground Rules

- Use `mise x -- <command> ...` for all project tools.
- Do not run `cargo`, `node`, `pnpm`, `npm`, or `tauri` directly.
- Keep frontend command/event names in sync with Rust command handlers and event constants.
- Preserve filesystem safety rules: no overwrites, validate roots, and prefer preserving source files over completing risky moves.
- Update specs/tasks only when the work is explicitly tied to a spec task.

## Code Guidelines

- Put UI state orchestration in `src/App.tsx` or focused React components.
- Put Tauri command wrappers and event subscriptions in `src/lib/api.ts`.
- Keep shared frontend payload types in `src/lib/types.ts` aligned with Rust `src-tauri/src/models.rs`.
- Put filesystem behavior in Rust modules, not the frontend.
- Add or update tests around path safety, conflict handling, history, metadata date selection, and queue behavior when changing those areas.

## Validation

Run the smallest meaningful check for the change, then broader checks before committing when practical:

```powershell
mise x -- pnpm test
mise x -- pnpm lint
mise x -- cargo test --manifest-path src-tauri/Cargo.toml
mise x -- pnpm build
```

## Commit Policy

Every change should be committed. Before committing:

```powershell
git status --short
```
