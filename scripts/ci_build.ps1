# Build java-runner2026 release binary, package dist/java-runner2026, and create zip artifact.
# Used locally and by .github/workflows/build.yml

param(
    [string]$DistName = "java-runner2026"
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Root = Split-Path -Parent $ScriptDir

Set-Location $Root

Write-Host "==> Installing Python dependencies"
python -m pip install --upgrade pip
python -m pip install -r requirements.txt

Write-Host "==> Building release binary (cargo build --release)"
cargo build --release

$DistDir = Join-Path (Join-Path $Root "dist") $DistName
Write-Host "==> Packaging release directory: $DistDir"
python scripts/package_release.py $DistDir

$ZipPath = Join-Path (Join-Path $Root "dist") "$DistName.zip"
if (Test-Path $ZipPath) {
    Remove-Item $ZipPath -Force
}

Write-Host "==> Creating zip artifact: $ZipPath"
Compress-Archive -Path $DistDir -DestinationPath $ZipPath

Write-Host ""
Write-Host "Build complete."
Write-Host "  Directory: $DistDir"
Write-Host "  Artifact:  $ZipPath"
