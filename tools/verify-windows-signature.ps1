[CmdletBinding()]
param(
    [Parameter(Mandatory, ValueFromPipeline)]
    [string[]]$InputPath,
    [switch]$RequireTimestamp
)

$ErrorActionPreference = 'Stop'
$results = foreach ($path in $InputPath) {
    $resolved = (Resolve-Path -LiteralPath $path).Path
    $signature = Get-AuthenticodeSignature -LiteralPath $resolved
    if ($signature.Status -ne [System.Management.Automation.SignatureStatus]::Valid) {
        throw "Authenticode verification failed for ${resolved}: $($signature.Status)"
    }
    if ($signature.SignerCertificate.EnhancedKeyUsageList.ObjectId -notcontains '1.3.6.1.5.5.7.3.3') {
        throw "Signer certificate does not allow code signing: $resolved"
    }
    if ($RequireTimestamp -and -not $signature.TimeStamperCertificate) {
        throw "Trusted timestamp is missing: $resolved"
    }
    [pscustomobject]@{
        path = $resolved
        status = $signature.Status.ToString()
        signer_subject = $signature.SignerCertificate.Subject
        signer_thumbprint = $signature.SignerCertificate.Thumbprint
        timestamped = [bool]$signature.TimeStamperCertificate
    }
}
$results | ConvertTo-Json -Depth 3
