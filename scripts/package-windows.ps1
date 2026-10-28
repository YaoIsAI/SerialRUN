param(
    [Parameter(Mandatory = $true)][string]$Version,
    [string]$Target = ""
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$binDir = if ($Target) { "target/$Target/release" } else { "target/release" }
$stage = "dist/SerialRUN-$Version-windows-x64"

Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $stage | Out-Null

Copy-Item "$binDir/serialrun.exe" $stage
if (Test-Path "$binDir/serialrun-cli.exe") { Copy-Item "$binDir/serialrun-cli.exe" $stage }
if (Test-Path "$binDir/serialrun-mcp.exe") { Copy-Item "$binDir/serialrun-mcp.exe" $stage }
Copy-Item "docs/help_en.md", "docs/help_zh.md", "docs/MANUAL.md", "docs/MANUAL_CN.md", "LICENSE" $stage

@"
SerialRUN v$Version (Windows x64)
=================================

Run serialrun.exe to start the GUI.
The package also includes:
    serialrun-cli.exe  - command-line interface (list, monitor, record, replay, agent)
    serialrun-mcp.exe  - standalone MCP server

Configuration and logs are stored in %USERPROFILE%\.serialrun\.
"@ | Set-Content -Encoding utf8 "$stage/INSTALL.txt"

$zip = Join-Path (Resolve-Path "dist") "serialrun-$Version-windows-x64.zip"
Remove-Item -Force $zip -ErrorAction SilentlyContinue
Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory(
    (Resolve-Path $stage).Path,
    $zip,
    [System.IO.Compression.CompressionLevel]::Optimal,
    $true
)

# Build the Inno Setup installer (.exe) if the compiler is available.
$iscc = Get-Command "ISCC.exe" -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source
if (-not $iscc) {
    $candidates = @(
        "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
    )
    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) { $iscc = $candidate; break }
    }
}

$setupExe = Join-Path (Resolve-Path "dist") "serialrun-$Version-windows-x64-setup.exe"
Remove-Item -Force $setupExe -ErrorAction SilentlyContinue

if ($iscc) {
    $repoRoot = (Resolve-Path ".").Path
    $stagePath = (Resolve-Path $stage).Path
    $distPath = (Resolve-Path "dist").Path
    & $iscc "/DMyAppVersion=$Version" "/DStageDir=$stagePath" "/DOutputDir=$distPath" "/DRepoRoot=$repoRoot" "scripts\windows-installer.iss"
    if ($LASTEXITCODE -ne 0) { throw "Inno Setup failed with exit code $LASTEXITCODE" }
    if (-not (Test-Path $setupExe)) { throw "Expected installer not found: $setupExe" }
    Write-Host "Created: $setupExe"
} elseif ($env:CI) {
    throw "ISCC.exe (Inno Setup) not found; cannot build the Windows installer in CI."
} else {
    Write-Warning "Inno Setup (ISCC.exe) not found - skipping installer. Install it from https://jrsoftware.org/isdl.php"
}

Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
Write-Host "Created: $zip"
