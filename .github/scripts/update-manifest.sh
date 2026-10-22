#!/bin/sh
# Prints latest.json, the file the desktop app reads to find and check its update.
# Run it in a folder that holds the .sig files of the release.
#   update-manifest.sh v1.2.3 owner/repo > latest.json
set -eu

tag=$1
repo=$2
version=${tag#v}
base="https://github.com/$repo/releases/download/$tag"

platform() {
  file="MediaOrganizer-$version-$2"
  jq -n --arg key "$1" --arg url "$base/$file" --rawfile signature "$file.sig" \
    '{($key): {url: $url, signature: $signature}}'
}

{
  platform windows-x86_64 windows-x64-setup.exe
  platform darwin-aarch64 macos-aarch64.app.tar.gz
  platform darwin-x86_64 macos-x86_64.app.tar.gz
} | jq -s --arg version "$version" --arg date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  '{version: $version, pub_date: $date, platforms: add}'
