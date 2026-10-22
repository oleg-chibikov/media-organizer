#!/bin/sh
# Stops a change to desktop logic the Android app copies when its Kotlin twin didn't change too.
# The pairs are in scripts/android-parity.map. A commit message line
# `Android-Parity: <why the app needs no change>` lets the change through.
#   check-android-parity.sh staged <commit message file>   from the commit-msg hook
#   check-android-parity.sh range <base> <head>             from CI
set -eu

cd "$(git rev-parse --show-toplevel)"
kotlin_root=android/app/src/
kotlin_package=java/com/mediaorganizer/android

case "${1:-}" in
  staged)
    changed=$(git diff --cached --name-only)
    message=$(cat "$2")
    ;;
  range)
    # A first push or a manual run has no base to compare with.
    if [ -z "${2:-}" ] || [ "$(printf '%s' "$2" | tr -d 0)" = "" ]; then exit 0; fi
    changed=$(git diff --name-only "$2" "$3")
    message=$(git log --format=%B "$2..$3")
    ;;
  *)
    echo "usage: $0 staged <message file> | range <base> <head>" >&2
    exit 2
    ;;
esac

if printf '%s\n' "$message" | grep -qi '^Android-Parity:'; then exit 0; fi

missing=""
while read -r desktop packages; do
  case "$desktop" in '' | '#'*) continue ;; esac
  desktop_changes=$(printf '%s\n' "$changed" | grep "^$desktop" | grep -v '/tests' || true)
  [ -n "$desktop_changes" ] || continue
  twin_changed=false
  for package in $(printf '%s' "$packages" | tr ',' ' '); do
    if printf '%s\n' "$changed" | grep -Eq "^$kotlin_root(main|test)/$kotlin_package/$package/"; then
      twin_changed=true
    fi
  done
  if [ "$twin_changed" = false ]; then
    missing="$missing$(printf '%s\n' "$desktop_changes" | sed "s|\$| -> android $packages|")
"
  fi
done < scripts/android-parity.map

[ -n "$missing" ] || exit 0
{
  echo "The Android app copies this desktop logic, and its Kotlin twin didn't change:"
  printf '%s' "$missing" | sed 's/^/  /'
  echo "Make the same change in the Kotlin package and its tests. docs/agent/android-parity.md says how."
  echo "If the app needs no change, add a line to the commit message: Android-Parity: <why>"
} >&2
exit 1
