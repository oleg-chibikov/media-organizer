# ExifTool Sidecar Setup

The app reads capture dates with ExifTool. Both copies below are committed, so a clone builds without extra steps. Keep both on the same ExifTool version.

## Windows

1. Download the Windows ExifTool zip from the official ExifTool website.
2. Copy `exiftool(-k).exe` into `src-tauri/binaries/` as `exiftool-x86_64-pc-windows-msvc.exe`.
3. Copy the `exiftool_files` folder next to it, into `src-tauri/binaries/exiftool_files/`.

Tauri needs that exact file name to bundle the sidecar for the Windows target. `tauri.windows.conf.json` puts `exiftool_files` next to the installed `exiftool.exe`, which can't start without it.

## macOS

1. Download the `Image-ExifTool-<version>.tar.gz` source package from the official ExifTool website.
2. Copy its `exiftool` script, `lib/` folder and `README` into `src-tauri/binaries/exiftool-perl/`.

The sidecars `exiftool-aarch64-apple-darwin` and `exiftool-x86_64-apple-darwin` are the same shell launcher. The launcher runs the script with the system Perl, which macOS ships in `/usr/bin/perl`. `tauri.macos.conf.json` puts `exiftool-perl/` into `Contents/Resources`, and the launcher looks there first.
