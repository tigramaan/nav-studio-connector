[CmdletBinding()]
param(
    [string[]]$InputPath = @(
        'src-tauri\target\release\nav-studio-connector.exe',
        'src-tauri\target\release\bundle\nsis\UMEC Nav Studio Connector_0.1.0_x64-setup.exe'
    )
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$temporaryRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("nav-connector-signing-hil-" + [guid]::NewGuid().ToString('N'))
$certificate = $null
$rootCertificate = $null
New-Item -ItemType Directory -Path $temporaryRoot | Out-Null
try {
    $certificate = New-SelfSignedCertificate -Type CodeSigningCert -Subject 'CN=UMEC Nav Studio Connector HIL' -CertStoreLocation 'Cert:\CurrentUser\My' -NotAfter (Get-Date).AddDays(2)
    $cerPath = Join-Path $temporaryRoot 'hil-code-signing.cer'
    Export-Certificate -Cert $certificate -FilePath $cerPath | Out-Null
    $rootCertificate = Import-Certificate -FilePath $cerPath -CertStoreLocation 'Cert:\CurrentUser\Root'
    $signedCopies = foreach ($path in $InputPath) {
        $resolved = (Resolve-Path -LiteralPath (Join-Path $repoRoot $path)).Path
        if (-not $resolved.StartsWith($repoRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Signing HIL input must remain inside the repository: $resolved"
        }
        $copy = Join-Path $temporaryRoot ([System.IO.Path]::GetFileName($resolved))
        Copy-Item -LiteralPath $resolved -Destination $copy
        $result = Set-AuthenticodeSignature -LiteralPath $copy -Certificate $certificate -HashAlgorithm SHA256
        if (-not $result.SignerCertificate -or $result.SignerCertificate.Thumbprint -ne $certificate.Thumbprint) {
            throw "HIL signature failed for ${copy}: $($result.Status) — $($result.StatusMessage)"
        }
        $copy
    }
    $verifier = Join-Path $PSScriptRoot 'verify-windows-signature.ps1'
    foreach ($copy in $signedCopies) {
        & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $verifier -InputPath $copy
        if ($LASTEXITCODE -ne 0) {
            throw "Fresh-process signature verification failed for ${copy}"
        }
        & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $verifier -InputPath $copy -RequireTimestamp *> $null
        if ($LASTEXITCODE -eq 0) {
            throw "Unsigned timestamp gate unexpectedly accepted ${copy}"
        }
    }
    [ordered]@{
        ok = $true
        signed_files = $signedCopies.Count
        disposable_certificate = $certificate.Thumbprint
        missing_timestamp_rejected = $true
    } | ConvertTo-Json -Compress
}
finally {
    if ($rootCertificate) { & certutil.exe -user -delstore Root $rootCertificate.Thumbprint *> $null }
    if ($certificate) { & certutil.exe -user -delstore My $certificate.Thumbprint *> $null }
    Remove-Item -LiteralPath $temporaryRoot -Recurse -Force -ErrorAction SilentlyContinue
}
