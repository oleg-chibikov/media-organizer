# Builds the Android APK on Windows 10 or 11 with only the built-in PowerShell and tar.
# JDK, Android SDK and Gradle download into android\.tools on the first run only.
#   powershell -ExecutionPolicy Bypass -File android\build-apk.ps1         builds android\dist\media-organizer.apk
#   powershell -ExecutionPolicy Bypass -File android\build-apk.ps1 test    also runs the unit tests
$ErrorActionPreference = "Stop"
# The progress bar makes Invoke-WebRequest many times slower in Windows PowerShell 5.1.
$ProgressPreference = "SilentlyContinue"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$jdkMajor = 17
$gradleVersion = "8.10.2"
$cmdlineToolsBuild = "15641748"
$platform = "android-35"
$buildTools = "35.0.0"

$projectDir = $PSScriptRoot
$toolsDir = Join-Path $projectDir ".tools"

function Get-Tool($url, $destination) {
    $zip = "$destination.zip"
    $temp = "$destination.tmp"
    Write-Host "Downloading $url"
    Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing
    if (Test-Path $temp) { Remove-Item $temp -Recurse -Force }
    New-Item -ItemType Directory $temp | Out-Null
    tar -xf $zip -C $temp
    if ($LASTEXITCODE -ne 0) { throw "Could not extract $zip" }
    # Each archive holds one top folder. Moving it last means a broken run leaves no half tool.
    Move-Item (Get-ChildItem $temp | Select-Object -First 1).FullName $destination
    Remove-Item $temp, $zip -Recurse -Force
}

New-Item -ItemType Directory -Force $toolsDir | Out-Null

$jdkDir = Join-Path $toolsDir "jdk-$jdkMajor"
if (-not (Test-Path $jdkDir)) {
    Get-Tool "https://api.adoptium.net/v3/binary/latest/$jdkMajor/ga/windows/x64/jdk/hotspot/normal/eclipse" $jdkDir
}
$env:JAVA_HOME = $jdkDir

$env:ANDROID_HOME = Join-Path $toolsDir "android-sdk"
# Gradle stops when an older ANDROID_SDK_ROOT points at another SDK.
$env:ANDROID_SDK_ROOT = $env:ANDROID_HOME
$cmdlineTools = Join-Path $env:ANDROID_HOME "cmdline-tools\latest"
$sdkmanager = Join-Path $cmdlineTools "bin\sdkmanager.bat"
if (-not (Test-Path $sdkmanager)) {
    New-Item -ItemType Directory -Force (Split-Path $cmdlineTools) | Out-Null
    if (Test-Path $cmdlineTools) { Remove-Item $cmdlineTools -Recurse -Force }
    Get-Tool "https://dl.google.com/android/repository/commandlinetools-win-${cmdlineToolsBuild}_latest.zip" $cmdlineTools
}
$platformDir = Join-Path $env:ANDROID_HOME "platforms\$platform"
$buildToolsDir = Join-Path $env:ANDROID_HOME "build-tools\$buildTools"
if (-not (Test-Path $platformDir) -or -not (Test-Path $buildToolsDir)) {
    ("y`n" * 20) | & $sdkmanager --licenses | Out-Null
    & $sdkmanager "platforms;$platform" "build-tools;$buildTools" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "sdkmanager could not install $platform and build-tools $buildTools" }
}

$gradleDir = Join-Path $toolsDir "gradle-$gradleVersion"
if (-not (Test-Path $gradleDir)) {
    Get-Tool "https://services.gradle.org/distributions/gradle-$gradleVersion-bin.zip" $gradleDir
}

& (Join-Path $gradleDir "bin\gradle.bat") --no-daemon --project-dir $projectDir @args assembleRelease
if ($LASTEXITCODE -ne 0) { throw "Gradle build failed" }

$distDir = Join-Path $projectDir "dist"
New-Item -ItemType Directory -Force $distDir | Out-Null
Copy-Item (Join-Path $projectDir "app\build\outputs\apk\release\app-release.apk") (Join-Path $distDir "media-organizer.apk")
Write-Host "Built $distDir\media-organizer.apk"
