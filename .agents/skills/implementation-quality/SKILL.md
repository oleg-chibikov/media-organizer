---
name: implementation-quality
description: Use when implementing, refactoring, reviewing, or polishing code in this repo where maintainability, performance, accessibility, readability, file purpose, or feature organization needs explicit scrutiny. Do not use for documentation-only edits.
---

# Implementation Quality Workflow

Use this skill as a focused quality pass after understanding the task and before final verification.

## Required Context

- Read `AGENTS.md`.
- Read `docs/agent/quickstart.md`.
- Read `docs/architecture.md` when behavior, filesystem work, command/event contracts, settings, history, or queues are involved.

## Quality Pass

- Confirm the change is allocated to the smallest coherent feature folder or module.
- Confirm each edited file has one purpose and no avoidable mixed responsibilities.
- Prefer deleting or simplifying code over adding abstractions.
- Check frontend changes for semantic HTML, keyboard access, labels, focus behavior, contrast, and avoidable render churn.
- Check backend changes for typed inputs/outputs, path safety, conflict handling, bounded work, and clear error propagation.
- Keep command/event/type changes synchronized across Rust and TypeScript.
- Add focused tests when behavior changes; otherwise state why tests are not applicable.

## Output Discipline

- Make the smallest code change that satisfies the task and the quality pass.
- Run the smallest meaningful verification command first, then broader checks when practical.
- Commit the final change.
