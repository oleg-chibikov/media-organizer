# Media Organizer

Windows and macOS desktop app that moves photos and videos into folders by year, month and day. It reads when each file was taken with a bundled ExifTool sidecar.

## Install

One command installs the latest release. No clone needed.

Windows, in PowerShell:

```powershell
irm https://raw.githubusercontent.com/oleg-chibikov/media-organizer/main/install.ps1 | iex
```

Windows, in Command Prompt (`cmd`):

```bat
powershell -NoProfile -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/oleg-chibikov/media-organizer/main/install.ps1 | iex"
```

macOS, in Terminal:

```sh
curl -fsSL https://raw.githubusercontent.com/oleg-chibikov/media-organizer/main/install.sh | sh
```

- **Windows** installs for the current user, no admin rights needed.
- **macOS** puts the app in `/Applications`, or `~/Applications` when `/Applications` is read-only.
- **Version** each script prints the version it installs. Release files carry it in their names.
- **Update** the app checks the latest release on start. It downloads it, installs it and restarts with no prompt.
- **During work** once a folder is picked the update waits for the next start, so a restart can't cut a move short.
- **Versions before the updater** need the install command once more.

## Logs

The app writes its log to the console window on Windows and to a file.

- **Windows** `%LOCALAPPDATA%\com.mediaorganizer.desktop\logs`
- **macOS** `~/Library/Logs/com.mediaorganizer.desktop`
- **What it holds** scans, moves, undo and redo steps, errors and ExifTool warnings.

The app window, the Windows console and the Android screen show the version in their title. Local builds show `0.2.0` on desktop and `dev` on Android.

## How The Parts Fit

```mermaid
flowchart TB
  person(["Person"])
  subgraph desktop["Desktop app, Windows and macOS"]
    direction LR
    fe["React frontend<br/>src/"]
    be["Rust backend<br/>src-tauri/src/"]
    exif[["ExifTool sidecar<br/>reads capture dates"]]
    fe -->|"commands: scan, plan, move, undo"| be
    be -.->|"events: progress per file"| fe
    be --> exif
  end
  subgraph phone["Android app, android/"]
    kt["Kotlin app<br/>organize/ holds the same rules"]
  end
  person -->|"on a computer"| fe
  person -->|"on the phone"| kt
  todo[("TODO folder,<br/>for example on a NAS")] --> be
  be --> dated[("Dated folders<br/>root/year/month/day")]
  gallery[("Phone gallery")] --> kt
  kt --> phoneDated[("Move to folder<br/>same year/month/day layout")]
  be -. "planner, filename and date rules,<br/>copied by hand" .-> kt
  release["release.yml<br/>one GitHub release"] -.->|installers| desktop
  release -.->|APK| phone
```

- **Desktop** the frontend only shows and asks. The backend scans, plans and moves files.
- **Android** runs alone on the phone and shares no code. A rule change in the backend needs the same change in `android/`.
- **Release** each push to `main` builds the Windows installer, the macOS DMGs and the APK into one release.

Each part has its own flow and code diagrams. The links are under Docs at the end of this page.

## Android

`android/` holds an Android app that moves the phone gallery into dated folders with the same rules. Get `MediaOrganizer-<version>-android.apk` from the [latest release](https://github.com/oleg-chibikov/media-organizer/releases/latest), or build it from scratch with no Android tools installed:

- **macOS or Linux** `./android/build-apk.sh`
- **Windows** `powershell -ExecutionPolicy Bypass -File android\build-apk.ps1`

The APK lands in `android/dist/media-organizer.apk`. Details are in `android/README.md`.

## Release

Each push to `main` publishes a new GitHub release. Pushes that change only `*.md`, `docs/` or `specs/` don't.

- **Version** the `Release` workflow raises the patch of the latest `vX.Y.Z` tag, for example `v0.2.0` to `v0.2.1`, and tags the commit.
- **Minor or major** run `gh workflow run release.yml -f bump=minor`, or `major`.
- **Assets** the Windows installer, a macOS DMG for each chip, and the signed Android APK.
- **Self update** each desktop build also ships a signed update file. `latest.json` points the app at them. The signing key sits in the `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets, and the public half in `tauri.conf.json`.
- **Early downloads** the release shows up as a pre-release right away. Each asset appears once its build ends. The release becomes latest when all builds pass.
- **Version in files** `package.json`, `Cargo.toml` and `tauri.conf.json` stay as they are. The build takes the version from the tag.

## Dependency updates

Dependabot opens update PRs each week for npm, Cargo, Gradle and GitHub Actions. The settings are in `.github/dependabot.yml`.

- **Auto-merge** `dependabot-auto-merge.yml` squashes a Dependabot PR once every check on it passes. No review is needed.
- **Failed checks** the PR stays open for a person to fix.
- **No release** the merge uses the workflow token, so it doesn't start `Release`. The next push to `main` ships the updates.

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
- Renames files where they are, with **Rename only** in the toolbar. Files keep their folder and get the dated name.
- Puts each photo and video in the folder of its place, found from GPS, with **Group by place** in the toolbar (on by default). A place abroad sits in its country folder. Each folder sits at the date level its files span: `2024/Japan/Kyoto/3/15` for months in Kyoto, `2024/Japan/3/Kyoto/15` for days of one month, `2024/Japan/3/15/Kyoto` for one day. A file with no GPS joins the place of a photo from the same camera within 3 hours. The Destination column waits for the places, so Move Files moves straight to them. **Rename only** turns it off.
- Shows the Country and Place of each file in the table, with a spinner while it looks. Double-click a place to see it on an OpenStreetMap map.
- Leaves files already in the destination where they are. Open the destination itself to lay them out again.
- Knows home, so places in home's country get no country folder. On first start with no home, the app reads the device location once on macOS and saves the nearest city. macOS asks for location access. Without access the app guesses home from the system time zone, such as Sydney for `Australia/Sydney`. A scan with 60 or more days in one city replaces that guess. Change it, clear it or press Use current location under Home in Settings. The Android app has no place folders, so it doesn't ask for location.

## User Flow

```mermaid
flowchart LR
  settings["Optional: picks a media root in Settings"] -.-> pick
  pick["Picks a TODO folder"] --> scan["Sees files stream into the table"]
  scan --> dates["Sees capture dates and planned names"]
  dates --> review["Checks rows that need review"]
  review --> confirm{"Confirms the move?"}
  confirm -->|no| dates
  confirm -->|yes| move["Watches files move into dated folders"]
  move -->|a check fails| stay["File stays put, the row says why"]
  move --> history["Opens History"]
  history --> undo{"Undo, redo or delete a run?"}
  undo -->|undo| back["Files return to the TODO folder"]
  undo -->|redo| move
```

The frontend and backend views of the same flow are in `src/README.md` and `src-tauri/README.md`.

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

Find unused code, as the `Unused code` CI workflow does:

```powershell
mise x -- pnpm knip
mise x -- cargo shear src-tauri
```

Rebuild the offline place table in `src-tauri/assets/geodata.tsv.gz` from GeoNames. The script downloads about 640 MB into `scripts/geodata/.cache`:

```powershell
python3 scripts/geodata/build_geodata.py
```

## Third-party data

Place names come from [GeoNames](https://www.geonames.org/), licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).

## Docs

- Architecture, and the commands and events between frontend and backend: `docs/architecture.md`
- Place folders, home and how a trip lifts its folder, every use case with diagrams: `docs/places.md`
- Frontend overview, flows and diagrams: `src/README.md`
- Backend overview, flows and diagrams: `src-tauri/README.md`
- ExifTool sidecar setup: `docs/exiftool-sidecar.md`
- Android app, flows and diagrams: `android/README.md`
- Agent conventions: `docs/agent/README.md`
- Feature specs: `specs/`
- Agent rules: `AGENTS.md`
