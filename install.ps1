# Installs the latest Media Organizer release on Windows for the current user:
#   irm https://raw.githubusercontent.com/oleg-chibikov/media-organizer/main/install.ps1 | iex
# The script block keeps settings and variables out of the caller's session.
& {
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'

    $repo = 'oleg-chibikov/media-organizer'
    $version = (Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest").tag_name.TrimStart('v')
    Write-Host "Installing Media Organizer $version"

    $asset = "MediaOrganizer-$version-windows-x64-setup.exe"
    $url = "https://github.com/$repo/releases/download/v$version/$asset"
    $installer = Join-Path ([System.IO.Path]::GetTempPath()) $asset

    Write-Host "Downloading $url"
    Invoke-WebRequest -Uri $url -OutFile $installer -UseBasicParsing

    try {
        $process = Start-Process -FilePath $installer -ArgumentList '/S' -Wait -PassThru
        if ($process.ExitCode -ne 0) {
            throw "The installer exited with code $($process.ExitCode)."
        }
    }
    finally {
        Remove-Item $installer -ErrorAction SilentlyContinue
    }

    Write-Host "Installed Media Organizer $version. Find it in the Start menu."
}
