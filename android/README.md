# Media Organizer for Android

Sorts the phone gallery into dated folders on the phone, with the same rules as the desktop app.

## Install

Download `MediaOrganizer-android.apk` from the [latest release](https://github.com/oleg-chibikov/media-organizer/releases/latest) on the phone and open it. Android asks once to allow installs from the browser.

## Build the APK from scratch

Clone the repo, then run one command from the repo folder. The script needs no Java, Android SDK or Android Studio.

macOS or Linux, in Terminal:

```sh
./android/build-apk.sh
```

Windows 10 or 11, in PowerShell or Command Prompt:

```bat
powershell -ExecutionPolicy Bypass -File android\build-apk.ps1
```

- **First run** downloads JDK 17, the Android SDK and Gradle into `android/.tools`, about 1 GB, and libraries into `~/.gradle`.
- **Later runs** reuse `android/.tools` and download only what is missing.
- **Output** is `android/dist/media-organizer.apk`, signed with the local debug key.
- **Tests** run when you add `test` to the command.

A locally built APK and a release APK have different keys. To switch between them, uninstall the app first.

## Use it

1. Open the app. The gallery folder starts as `/storage/emulated/0/DCIM/Camera`, and "Move to" as `<gallery folder>/Processed`.
2. Press Choose next to a field to pick another folder, or type a full path. The app keeps both folders after it closes.
3. Press Start. The first time, Android asks for access to all files. Allow it and press Start again. Choose asks the same way.
4. Keep the app on screen until the line under Start says Done. Android pauses a backgrounded app.

Choose next to "Move to" creates the folder first when it doesn't exist yet, so the picker opens inside it.

## What matches the desktop app

- **Folders** `<root>/<year>/<month>/<day>/<date> (<original>) - <device>.<ext>`.
- **Dates** come from the same tags in the same order, ending with the file date.
- **Conflicts** get a ` (n)` suffix. A file with the same bytes already there stays in place.
- **Skipped** files are anything under `Processed`, the destination, and non-media files.
- **Cleanup** removes source folders the run emptied, but keeps the gallery folder.

## What differs

- **Metadata** comes from Android's `ExifInterface` and `MediaMetadataRetriever` instead of ExifTool.
- **Video device name** is missing. Android doesn't expose the camera model of a video.
- **Hidden files** such as `.trashed-*` and `.pending-*` stay put. They belong to the Android gallery.
- **History and undo** don't exist. The log on screen shows the last 300 moves.

The sorting code is a copy of `src-tauri/src/planner`, `filename` and `metadata/date_selection.rs`. A rule change there needs the same change in `app/src/main/java/com/mediaorganizer/android/organize`.

## Release signing

The `Release` workflow signs the APK with the key in the `ANDROID_KEYSTORE_BASE64` and `ANDROID_KEYSTORE_PASSWORD` repo secrets. One key for every release lets a new APK install over the old one. A new key means uninstalling the app once.
