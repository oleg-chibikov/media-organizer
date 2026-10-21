# Media Organizer

Windows and macOS desktop app for organizing photo and video TODO folders into date-based destinations using metadata from a bundled ExifTool sidecar.

## Install

One command installs the latest release. No clone needed.

Windows, in PowerShell:

```powershell
irm https://raw.githubusercontent.com/oleg-chibikov/media-organizer/main/install.ps1 | iex
```

Windows, in Command Prompt (`cmd`), with the `curl` that Windows 10 and 11 include:

```bat
curl -fL -o "%TEMP%\MediaOrganizer-setup.exe" https://github.com/oleg-chibikov/media-organizer/releases/latest/download/MediaOrganizer-windows-x64-setup.exe && start /wait "" "%TEMP%\MediaOrganizer-setup.exe" /S && del "%TEMP%\MediaOrganizer-setup.exe"
```

macOS, in Terminal:

```sh
curl -fsSL https://raw.githubusercontent.com/oleg-chibikov/media-organizer/main/install.sh | sh
```

- **Windows** installs for the current user, no admin rights needed.
- **macOS** puts the app in `/Applications`, or `~/Applications` when `/Applications` is read-only.
- **Update** by running the same command again.

## Android

`android/` holds an Android app that sorts the phone gallery with the same rules. Get `MediaOrganizer-android.apk` from the [latest release](https://github.com/oleg-chibikov/media-organizer/releases/latest), or build it from scratch with no Android tools installed:

- **macOS or Linux** `./android/build-apk.sh`
- **Windows** `powershell -ExecutionPolicy Bypass -File android\build-apk.ps1`

The APK lands in `android/dist/media-organizer.apk`. Details are in `android/README.md`.

## Release

Push a version tag, for example `git tag v0.2.0 && git push origin v0.2.0`. The `Release` workflow builds the Windows installer, the macOS apps and the Android APK, then publishes them as the latest GitHub release.

## What It Does

- Scans a selected TODO folder recursively.
- Skips files already under `Processed/`.
- Reads photo/video capture dates from metadata, not filename dates.
- Streams destination previews as metadata becomes available.
- Moves only after explicit confirmation.
- Avoids overwrites with conflict suffixes and execution-time conflict checks.
- Falls back from rename to copy-verify-keep-source for NAS/cross-device behavior.
- Stores processing history in appData for inspection, undo, and redo.
- Supports either legacy `<selected folder>/Processed` destinations or a persisted custom media root.

## Stack

- Frontend: React, TypeScript, Vite, Tailwind CSS.
- Desktop shell: Tauri 2.
- Backend: Rust.
- Metadata: bundled ExifTool sidecar.
- Tooling: `mise`, `pnpm`, `cargo`.

## Development

All project tools must be run through `mise`.

```powershell
mise x -- pnpm install
mise x -- pnpm tauri dev
```

Useful checks:

```powershell
mise x -- pnpm test
mise x -- pnpm lint
mise x -- cargo test --manifest-path src-tauri/Cargo.toml
mise x -- pnpm build
```

## Docs

- Architecture: `docs/architecture.md`
- Frontend overview, flows and diagrams: `src/README.md`
- Backend overview, flows and diagrams: `src-tauri/README.md`
- ExifTool sidecar setup: `docs/exiftool-sidecar.md`
- Android app: `android/README.md`
- Agent conventions and upstream references: `docs/agent/README.md`
- Feature specs: `specs/`
- Agent rules: `AGENTS.md`
