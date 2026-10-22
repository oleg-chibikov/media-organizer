# Keeping the Android App in Step with Desktop

The Android app in `android/` copies desktop rules into Kotlin. A change to one of those rules on desktop needs the same change in the app, in the same commit.

## Which code has a twin

`scripts/android-parity.map` pairs each desktop path with the Kotlin packages that copy it. Those packages sit in `android/app/src/main/java/com/mediaorganizer/android/`:

- **`geo/`** copies `src-tauri/src/geo`. Both read the tables in `src-tauri/assets`.
- **`places/`** copies `file_places`, `home_location` and `settings/home_precedence.rs`, and the place parts of `planner`: levels, existing folders, nearby days and library moves.
- **`organize/`** copies the dated part of `planner`, `filename`, the date rules of `metadata` and the scan rules of `scanner`.
- **`metadata/`** copies the GPS rules of `metadata/gps.rs`.
- **`location/`** is the Android side of `home_location/device`: location fix, time zone, saved home.

## How to carry a change over

`docs/places.md` lists every place use case with the desktop and Android tests that prove it. A changed rule changes its row there too.

1. Change the desktop rule and its Rust test.
2. Find the twin through `scripts/android-parity.map`, and make the same change in Kotlin.
3. Port the new or changed Rust test case. The parity tests keep the desktop names and points: `FilePlacesParityTest`, `HomeParityTest`, `NearbyFoldersTest`, `PlaceFoldersTest`, `GeocoderTest`.
4. Run `mise x -- cargo test --manifest-path src-tauri/Cargo.toml` and `./android/build-apk.sh test`.
5. Commit both sides together.

The commit-msg hook and the `android-parity` job in `checks.yml` run `scripts/check-android-parity.sh`. It stops a commit that changes a mapped desktop path without its Kotlin package. A desktop change that the app doesn't need, such as ExifTool workers or Tauri events, goes through with a commit message line `Android-Parity: <why>`.

A new desktop module that the app copies gets a line in the map.

## Where the app differs on purpose

- **One pass.** The app reads every date, finds places, moves earlier days, then moves files. It has no previews, no `Reservations` and no undo. A name is checked free right before each move.
- **Earlier days always gather.** Desktop can keep them apart for one scan through `set_gather_library`. The app has no such box.
- **Scan rules.** The app also skips hidden files and the Move to folder. `organize/isScanned` says which files a run holds, so library moves leave them out, as `planner/library_moves.rs` does.
- **Home on launch.** The app looks for a device home when the screen opens, with Android's approximate location. Background runs only read the saved home.
- **Photos must survive.** Each move renames into a free name, or keeps the file where it is. `TripTest` stops a run at every step and checks every file is still there. Keep it passing.
