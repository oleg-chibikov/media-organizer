# Agent Guidance

This folder keeps repo-specific guidance short enough for agents to load only what they need.

## Context Placement

- `AGENTS.md`: always-on rules for tooling, workflow, repo invariants, quality, and verification.
- `docs/agent/quickstart.md`: compact project conventions for day-to-day code changes.
- `docs/overview.md`: the app on one page, for a first read.
- `docs/features/`: one page per feature with user flows, sequences and code paths.
- `docs/places.md`: the place, home and trip rules of both apps, with diagrams and every use case.
- `docs/agent/android-parity.md`: which desktop code the Android app copies, and how to carry a change over.
- `docs/architecture.md`: system behavior, data flow, backend/frontend boundaries, and safety invariants.
- `.agents/skills/`: repeatable workflows that should trigger only for matching tasks.

## Usage order for agents

1. `AGENTS.md`
2. `docs/agent/quickstart.md`
3. `docs/architecture.md` when changing behavior or cross-boundary contracts
4. A repo skill only when its description matches the task

## What Belongs Where

- Put permanent non-negotiable rules in `AGENTS.md`.
- Put explanatory project context and examples in docs.
- Put specialized, repeatable procedures in skills.
- Do not duplicate long docs inside skills; point to the relevant concise doc instead.
- Do not hide universal code quality rules inside skills, because skill activation is conditional.

## Skill Authoring Rules

- Use a clear `description` that states when the skill should and should not trigger.
- Keep `SKILL.md` focused on the workflow.
- Use `references/` only for optional details.
- Use `scripts/` only for deterministic repetitive work.
