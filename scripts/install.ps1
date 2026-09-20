$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $architecture = if ([Environment]::Is64BitOperatingSystem) { 'x86_64' } else { 'i686' }
    $rustup = Join-Path $env:TEMP 'rustup-init.exe'
    Invoke-WebRequest "https://win.rustup.rs/$architecture" -OutFile $rustup
    & $rustup -y
    Remove-Item $rustup -Force
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
}

cargo build --release
$installDir = if ($env:TOON_WORLD_INSTALL_DIR) { $env:TOON_WORLD_INSTALL_DIR } else { Join-Path $env:USERPROFILE '.local\bin' }
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Copy-Item target\release\toon-world.exe (Join-Path $installDir 'toon-world.exe') -Force

if (($env:PATH -split ';') -notcontains $installDir) {
    Write-Output "Installed to $installDir. Add it to PATH to run toon-world from any directory."
}
