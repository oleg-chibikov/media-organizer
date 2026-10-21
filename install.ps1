# Installs the latest Media Organizer release on Windows for the current user:
#   irm https://raw.githubusercontent.com/oleg-chibikov/media-organizer/main/install.ps1 | iex
# The script block keeps settings and variables out of the caller's session.
& {
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'

    $url = 'https://github.com/oleg-chibikov/media-organizer/releases/latest/download/MediaOrganizer-windows-x64-setup.exe'
    $installer = Join-Path ([System.IO.Path]::GetTempPath()) 'MediaOrganizer-setup.exe'

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

    Write-Host 'Installed Media Organizer. Find it in the Start menu.'
}
