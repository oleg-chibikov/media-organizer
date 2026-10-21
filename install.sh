#!/bin/sh
# Installs the latest Media Organizer release on macOS:
#   curl -fsSL https://raw.githubusercontent.com/oleg-chibikov/media-organizer/main/install.sh | sh
set -eu

repo="oleg-chibikov/media-organizer"
app_name="Media Organizer.app"

if [ "$(uname -s)" != "Darwin" ]; then
  echo "This script installs the macOS app. On Windows run install.ps1." >&2
  exit 1
fi

# A Rosetta shell reports x86_64 on Apple Silicon, so ask the hardware.
if [ "$(sysctl -n hw.optional.arm64 2>/dev/null || echo 0)" = "1" ]; then
  arch="aarch64"
else
  arch="x86_64"
fi

if [ -w /Applications ]; then
  install_dir="/Applications"
else
  install_dir="$HOME/Applications"
  mkdir -p "$install_dir"
fi

work_dir=$(mktemp -d)
trap 'rm -rf "$work_dir"' EXIT

url="https://github.com/$repo/releases/latest/download/MediaOrganizer-macos-$arch.app.tar.gz"
echo "Downloading $url"
curl -fsSL "$url" | tar -xz -C "$work_dir"

rm -rf "${install_dir:?}/$app_name"
mv "$work_dir/$app_name" "$install_dir/"
# The app is not notarized, so Gatekeeper would block it if anything marked it as downloaded.
xattr -dr com.apple.quarantine "$install_dir/$app_name" 2>/dev/null || true

echo "Installed $install_dir/$app_name"
