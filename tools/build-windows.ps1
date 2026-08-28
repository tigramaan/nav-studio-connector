[CmdletBinding()]
param(
    [switch]$DryRun,
    [switch]$SkipInstall
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
if (-not (Test-Path -LiteralPath (Join-Path $repoRoot 'src-tauri\Cargo.toml'))) {
    throw 'Run this tool from the Nav Studio Connector repository'
}

$commands = @()
if (-not $SkipInstall) { $commands += ,@('npm', @('ci', '--no-audit', '--no-fund')) }
$commands += ,@('npm', @('run', 'build'))
$commands += ,@('cargo', @('fmt', '--manifest-path', 'src-tauri/Cargo.toml', '--', '--check'))
$commands += ,@('cargo', @('test', '--manifest-path', 'src-tauri/Cargo.toml'))
$commands += ,@('npm', @('run', 'validate:traceability'))
$commands += ,@('node', @('tools/scan-secrets.mjs'))
$commands += ,@('npm', @('run', 'tauri', '--', 'build', '--bundles', 'nsis'))

if ($DryRun) {
    $commands | ForEach-Object { "{0} {1}" -f $_[0], ($_[1] -join ' ') }
    exit 0
}

Push-Location -LiteralPath $repoRoot
try {
    foreach ($entry in $commands) {
        & $entry[0] @($entry[1])
        if ($LASTEXITCODE -ne 0) { throw "Command failed: $($entry[0])" }
    }
}
finally {
    Pop-Location
}
