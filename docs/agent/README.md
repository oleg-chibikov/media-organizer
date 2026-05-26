# Agent Standards Pack

This folder is a low-token, repo-local standards setup for coding agents.

## What this gives you

- A short project-specific operating guide: `quickstart.md`.
- Vendored upstream docs from established sources under `upstream/`.
- A one-command refresh script so docs can be updated without manual copy/paste.

## Usage order for agents

1. `AGENTS.md`
2. `docs/agent/quickstart.md`
3. Only the relevant file(s) in `docs/agent/upstream/`

## Refresh upstream references

```powershell
powershell -File scripts/sync-agent-docs.ps1
```

## Why this is token-efficient

- Most tasks should only need `quickstart.md`.
- Upstream docs are split by technology so agents can read only one file per task area.
- Source URLs are tracked in `upstream/SOURCES.md`.
