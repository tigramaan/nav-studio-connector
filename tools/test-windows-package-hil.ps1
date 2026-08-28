[CmdletBinding()]
param(
    [string]$InstallerPath = 'src-tauri\target\release\bundle\nsis\UMEC Nav Studio Connector_0.1.0_x64-setup.exe'
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$installer = (Resolve-Path -LiteralPath (Join-Path $repoRoot $InstallerPath)).Path
if (-not $installer.StartsWith($repoRoot + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'Package HIL installer must remain inside the repository'
}

$uninstallRoot = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall'
$displayName = 'UMEC Nav Studio Connector'
function Get-ConnectorRegistration {
    Get-ChildItem -LiteralPath $uninstallRoot -ErrorAction SilentlyContinue |
        Where-Object { (Get-ItemProperty -LiteralPath $_.PSPath -ErrorAction SilentlyContinue).DisplayName -eq $displayName } |
        Select-Object -First 1
}

function Get-RegisteredUninstaller([object]$Properties) {
    $command = [string]$Properties.UninstallString
    if ($command -match '^"([^"\r\n]+\.exe)"(?:\s|$)') { return $Matches[1] }
    if ($command -match '^([^"\r\n]+\.exe)$') { return $Matches[1].Trim() }
    throw 'Registered uninstall command is malformed'
}

if (Get-ConnectorRegistration) {
    throw 'Refusing package HIL because a pre-existing connector installation is registered'
}

$installedByHil = $false
$registration = $null
$gui = $null
try {
    $install = Start-Process -FilePath $installer -ArgumentList '/S' -Wait -PassThru
    if ($install.ExitCode -ne 0) { throw "Silent installer returned $($install.ExitCode)" }
    $installedByHil = $true
    $registration = Get-ConnectorRegistration
    if (-not $registration) { throw 'Silent installer created no current-user registration' }
    $properties = Get-ItemProperty -LiteralPath $registration.PSPath
    $installLocation = ([string]$properties.InstallLocation).Trim('"')
    if (-not $installLocation) {
        $installLocation = Split-Path -Parent (Get-RegisteredUninstaller $properties)
    }
    if (-not (Test-Path -LiteralPath $installLocation -PathType Container)) {
        throw 'Registered install location is missing'
    }
    $application = Get-ChildItem -LiteralPath $installLocation -Filter '*.exe' |
        Where-Object Name -NotMatch 'uninstall' |
        Select-Object -First 1
    if (-not $application) { throw 'Installed connector executable is missing' }
    $description = & $application.FullName agent describe --json | ConvertFrom-Json
    if (-not $description.ok -or $description.operation -ne 'agent.describe') {
        throw 'Installed CLI contract smoke failed'
    }
    $gui = Start-Process -FilePath $application.FullName -PassThru
    if ($gui.WaitForExit(5000)) {
        throw "Installed GUI exited during smoke interval with $($gui.ExitCode)"
    }
    Stop-Process -Id $gui.Id -Force
    $gui = $null
    [ordered]@{
        ok = $true
        installer = $installer
        installed_executable = $application.FullName
        cli_smoke = $true
        gui_smoke_seconds = 5
    } | ConvertTo-Json -Compress
}
finally {
    if ($gui -and -not $gui.HasExited) { Stop-Process -Id $gui.Id -Force -ErrorAction SilentlyContinue }
    if ($installedByHil) {
        $registration = Get-ConnectorRegistration
        if ($registration) {
            $properties = Get-ItemProperty -LiteralPath $registration.PSPath
            $uninstaller = Get-RegisteredUninstaller $properties
            if ($uninstaller -and (Test-Path -LiteralPath $uninstaller -PathType Leaf)) {
                Start-Process -FilePath $uninstaller -ArgumentList '/S' -Wait | Out-Null
            }
        }
    }
}

if (Get-ConnectorRegistration) {
    throw 'Connector registration remained after package HIL cleanup'
}
