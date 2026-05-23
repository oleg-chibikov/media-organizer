# ExifTool Sidecar Setup (Windows)

1. Download ExifTool for Windows from the official ExifTool website.
2. Extract the archive and locate the executable.
3. Copy the executable into:

   `src-tauri/binaries/`

4. Rename it to:

   `exiftool-x86_64-pc-windows-msvc.exe`

This filename is required so Tauri can bundle and resolve the sidecar for the Windows target.
