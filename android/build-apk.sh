#!/bin/sh
# Builds the Android APK on macOS or Linux with only curl, unzip and tar installed.
# JDK, Android SDK and Gradle download into android/.tools on the first run only.
#   ./android/build-apk.sh          builds android/dist/media-organizer.apk
#   ./android/build-apk.sh test     also runs the unit tests
set -eu

cd "$(dirname "$0")"
project_dir=$PWD
tools_dir="$project_dir/.tools"

jdk_major=17
gradle_version=9.6.0
cmdline_tools_build=15641748
platform=android-35
build_tools=36.0.0

case "$(uname -s)" in
  Darwin) jdk_os=mac; sdk_os=mac ;;
  Linux) jdk_os=linux; sdk_os=linux ;;
  *) echo "Run this on macOS or Linux." >&2; exit 1 ;;
esac

# A Rosetta shell reports x86_64 on Apple Silicon, so ask the hardware.
if [ "$(sysctl -n hw.optional.arm64 2>/dev/null || echo 0)" = "1" ]; then
  jdk_arch=aarch64
else
  case "$(uname -m)" in
    arm64 | aarch64) jdk_arch=aarch64 ;;
    *) jdk_arch=x64 ;;
  esac
fi

download() {
  echo "Downloading $1"
  curl -fL --retry 3 --progress-bar -o "$2" "$1"
}

mkdir -p "$tools_dir"

jdk_dir="$tools_dir/jdk-$jdk_major"
if [ ! -d "$jdk_dir" ]; then
  download "https://api.adoptium.net/v3/binary/latest/$jdk_major/ga/$jdk_os/$jdk_arch/jdk/hotspot/normal/eclipse" "$tools_dir/jdk.tar.gz"
  rm -rf "$jdk_dir.tmp"
  mkdir -p "$jdk_dir.tmp"
  tar -xzf "$tools_dir/jdk.tar.gz" -C "$jdk_dir.tmp" --strip-components=1
  mv "$jdk_dir.tmp" "$jdk_dir"
  rm "$tools_dir/jdk.tar.gz"
fi
if [ -d "$jdk_dir/Contents/Home" ]; then
  JAVA_HOME="$jdk_dir/Contents/Home"
else
  JAVA_HOME="$jdk_dir"
fi
export JAVA_HOME

ANDROID_HOME="$tools_dir/android-sdk"
# Gradle stops when an older ANDROID_SDK_ROOT points at another SDK.
ANDROID_SDK_ROOT="$ANDROID_HOME"
export ANDROID_HOME ANDROID_SDK_ROOT
sdkmanager="$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"
if [ ! -x "$sdkmanager" ]; then
  download "https://dl.google.com/android/repository/commandlinetools-$sdk_os-${cmdline_tools_build}_latest.zip" "$tools_dir/cmdline-tools.zip"
  rm -rf "$tools_dir/cmdline-tools" "$ANDROID_HOME/cmdline-tools/latest"
  unzip -q "$tools_dir/cmdline-tools.zip" -d "$tools_dir"
  mkdir -p "$ANDROID_HOME/cmdline-tools"
  mv "$tools_dir/cmdline-tools" "$ANDROID_HOME/cmdline-tools/latest"
  rm "$tools_dir/cmdline-tools.zip"
fi
if [ ! -d "$ANDROID_HOME/platforms/$platform" ] || [ ! -d "$ANDROID_HOME/build-tools/$build_tools" ]; then
  yes | "$sdkmanager" --licenses >/dev/null
  "$sdkmanager" "platforms;$platform" "build-tools;$build_tools" >/dev/null
fi

gradle_dir="$tools_dir/gradle-$gradle_version"
if [ ! -d "$gradle_dir" ]; then
  download "https://services.gradle.org/distributions/gradle-$gradle_version-bin.zip" "$tools_dir/gradle.zip"
  rm -rf "$gradle_dir.tmp"
  unzip -q "$tools_dir/gradle.zip" -d "$gradle_dir.tmp"
  mv "$gradle_dir.tmp/gradle-$gradle_version" "$gradle_dir"
  rm -rf "$gradle_dir.tmp" "$tools_dir/gradle.zip"
fi

"$gradle_dir/bin/gradle" --no-daemon --project-dir "$project_dir" "$@" assembleRelease

mkdir -p "$project_dir/dist"
cp "$project_dir/app/build/outputs/apk/release/app-release.apk" "$project_dir/dist/media-organizer.apk"
echo "Built $project_dir/dist/media-organizer.apk"
