# Quickstart Conventions

Use this file as the default conventions contract for AI-assisted changes in this repo.

## Architecture Guardrails

- Rust backend is the source of truth for behavior and filesystem decisions.
- React renders state and orchestrates requests/events; it must not invent filesystem outcomes.
- Keep command/event contracts synced across:
  - `src-tauri/src/models/`
  - `src-tauri/src/commands/` and `src-tauri/src/events.rs`
  - `src/lib/types/` and `src/lib/api/`
- Follow the layering in `docs/architecture.md`. ESLint enforces the frontend part.

## React + TypeScript

- Keep components pure where possible; avoid unnecessary effects.
- Prefer explicit types at command boundaries and event payload boundaries.
- Keep cross-feature wiring in `src/app/` and layout in `src/App.tsx`; avoid hidden shared mutable state.
- Derive tallies from state you already hold instead of keeping a second counter.
- Keep UI accessible by default: semantic elements, clear labels, keyboard operation, visible focus, and usable contrast.
- Avoid unnecessary re-renders by keeping derived state local and not duplicating backend state.
- React Compiler (`babel-plugin-react-compiler` in `vite.config.ts`) memoizes components and hooks. Skip new `useMemo`, `useCallback` and `React.memo` unless a value must stay stable for an effect.

## File And Feature Shape

- Keep every file single-purpose and short enough to scan.
- Put multi-file frontend features in `src/features/<feature>/`.
- Keep reusable primitives in `src/components/design-system/` and feature-specific components near the feature.
- Import another feature only through its `index.ts`.
- Put shared command/event bridge code in `src/lib/api/` and shared payload types in `src/lib/types/`.
- Prefer backend folder modules when a Rust capability grows beyond one cohesive file.
- Do not create broad utility files; name helpers after the domain decision they own.

## Tailwind + Vite

- Use static Tailwind class names whenever possible so class detection remains reliable.
- Keep dynamic styling constrained to clearly bounded variants.
- Keep environment usage explicit through Vite conventions (`import.meta.env`).

## Rust + Tauri

- Prefer typed command inputs/outputs with `serde` models.
- Make filesystem commands `async` and run the work through `commands/background.rs`.
- Emit events through `EventSink`; test streams with `test_support::events::RecordingSink`.
- Return `String` errors at the command boundary; add an error enum only where code branches on the kind.
- Do not use `unwrap`/`expect` in command paths unless failure is intentionally fatal and documented.
- Keep path safety checks in `path_safety.rs`. Rename, copy and delete only through `safe_fs/`; clippy rejects raw `std::fs` mutations elsewhere.

## Validation Strategy

Run the smallest meaningful check first, then expand:

```powershell
mise x -- pnpm test
mise x -- pnpm lint
mise x -- cargo test --manifest-path src-tauri/Cargo.toml
mise x -- pnpm build
```

The `Checks` CI workflow runs these on every pull request, plus `pnpm format:check`, `cargo fmt --check` and `pnpm lint:rust`.

Find unused code, as the `Unused code` CI workflow does:

```powershell
mise x -- pnpm knip
mise x -- cargo shear src-tauri
```

## Commit Quality

- Use Conventional Commits when possible.
- Keep commits scoped to one logical change.
- If behavior changes, include tests or justify why tests are not applicable.
