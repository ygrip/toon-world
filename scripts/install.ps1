param(
    [string]$Version = $(if ($env:TOON_WORLD_VERSION) { $env:TOON_WORLD_VERSION } else { 'latest' }),
    [string]$InstallDir = $(if ($env:TOON_WORLD_INSTALL_DIR) { $env:TOON_WORLD_INSTALL_DIR } else { Join-Path $HOME '.local\bin' })
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repo = 'ygrip/toon-world'
$baseReleaseUrl = "https://github.com/$repo/releases"

$arch = switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture) {
    'X64' { 'x86_64' }
    'Arm64' { 'aarch64' }
    default { throw "Unsupported architecture: $([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture)" }
}

$asset = "toon-world-windows-$arch.exe"

$releaseBase = if ([string]::IsNullOrWhiteSpace($Version) -or $Version -eq 'latest') {
    "$baseReleaseUrl/latest/download"
} elseif ($Version.StartsWith('v')) {
    "$baseReleaseUrl/download/$Version"
} else {
    "$baseReleaseUrl/download/v$Version"
}

$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("toon-world-install-" + [Guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

try {
    $binaryPath = Join-Path $tempDir $asset
    $checksumsPath = Join-Path $tempDir 'SHA256SUMS'

    Write-Output "Detected Windows $arch -> $asset"
    Write-Output 'Downloading toon-world...'

    Invoke-WebRequest "$releaseBase/$asset" -OutFile $binaryPath
    Invoke-WebRequest "$releaseBase/SHA256SUMS" -OutFile $checksumsPath

    $checksumLine = Get-Content $checksumsPath |
        Where-Object {
            $parts = ($_ -split '\s+')
            $parts.Count -ge 2 -and $parts[-1].TrimStart('*') -eq $asset
        } |
        Select-Object -First 1

    if (-not $checksumLine) {
        throw "Checksum for $asset was not found in SHA256SUMS."
    }

    $expected = (($checksumLine -split '\s+')[0]).ToLowerInvariant()
    $actual = (Get-FileHash -Path $binaryPath -Algorithm SHA256).Hash.ToLowerInvariant()

    if ($actual -ne $expected) {
        throw "Checksum mismatch for $asset. Expected $expected, got $actual."
    }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    $destination = Join-Path $InstallDir 'toon-world.exe'
    $tempDestination = Join-Path $InstallDir '.toon-world-install.exe'
    Copy-Item $binaryPath $tempDestination -Force
    Move-Item $tempDestination $destination -Force

    Write-Output "Installed toon-world to $destination"

    $pathEntries = $env:PATH -split ';'
    if ($pathEntries -notcontains $InstallDir) {
        Write-Output "Add $InstallDir to PATH to run toon-world from any directory."
    }
}
finally {
    Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
}
