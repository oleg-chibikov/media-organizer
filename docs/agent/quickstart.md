# Quickstart Conventions

Use this file as the default conventions contract for AI-assisted changes in this repo.

## Architecture Guardrails

- Rust backend is the source of truth for behavior and filesystem decisions.
- React renders state and orchestrates requests/events; it must not invent filesystem outcomes.
- Keep command/event contracts synced across:
  - `src-tauri/src/models.rs`
  - `src-tauri/src/commands.rs` and `src-tauri/src/events.rs`
  - `src/lib/types.ts` and `src/lib/api.ts`

## React + TypeScript

- Keep components pure where possible; avoid unnecessary effects.
- Prefer explicit types at command boundaries and event payload boundaries.
- Keep orchestration in `src/App.tsx` and focused components; avoid hidden shared mutable state.

## Tailwind + Vite

- Use static Tailwind class names whenever possible so class detection remains reliable.
- Keep dynamic styling constrained to clearly bounded variants.
- Keep environment usage explicit through Vite conventions (`import.meta.env`).

## Rust + Tauri

- Prefer typed command inputs/outputs with `serde` models.
- Do not use `unwrap`/`expect` in command paths unless failure is intentionally fatal and documented.
- Keep path safety checks centralized in backend modules; preserve no-overwrite guarantees.

## Validation Strategy

Run the smallest meaningful check first, then expand:

```powershell
mise x -- pnpm test
mise x -- pnpm lint
mise x -- cargo test --manifest-path src-tauri/Cargo.toml
mise x -- pnpm build
```

## Commit Quality

- Use Conventional Commits when possible.
- Keep commits scoped to one logical change.
- If behavior changes, include tests or justify why tests are not applicable.

## Deeper References

Use `docs/agent/upstream/` for canonical docs from React, Tailwind, Vite, Tauri, Rust API Guidelines, and commit/changelog specs.
