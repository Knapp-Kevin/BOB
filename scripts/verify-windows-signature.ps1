# Fails unless every given file carries a valid, timestamped MythologIQ Labs, LLC
# Authenticode signature chaining to Microsoft Identity Verification Root Certificate
# Authority 2020. These are the release checks from the Windows code signing standard
# (MythologIQ-Labs-LLC/Myth-Tech-Forge docs/WINDOWS_CODE_SIGNING_STANDARD.md).
# Read-only: it changes no file, certificate store or security setting.
param(
    [Parameter(Mandatory)] [string[]] $Path,
    [string] $SignToolPath = $env:BOB_SIGNTOOL
)
$ErrorActionPreference = 'Stop'

if (-not $SignToolPath) {
    $SignToolPath = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
        Sort-Object FullName -Descending | Select-Object -First 1 -ExpandProperty FullName
}
if (-not $SignToolPath -or -not (Test-Path -LiteralPath $SignToolPath)) { throw 'signtool.exe not found; install the Windows SDK or pass -SignToolPath' }

foreach ($file in $Path) {
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "nothing to verify at $file" }

    & $SignToolPath verify /pa /v $file
    if ($LASTEXITCODE -ne 0) { throw "signtool verify /pa failed for $file" }

    $sig = Get-AuthenticodeSignature -LiteralPath $file
    if ($sig.Status -ne 'Valid') { throw "$file Authenticode status is $($sig.Status), not Valid" }
    $signer = $sig.SignerCertificate.GetNameInfo([Security.Cryptography.X509Certificates.X509NameType]::SimpleName, $false)
    if ($signer -cne 'MythologIQ Labs, LLC') { throw "$file is signed by '$signer', not 'MythologIQ Labs, LLC'" }
    if (-not $sig.TimeStamperCertificate) { throw "$file signature is not timestamped" }

    $chain = [Security.Cryptography.X509Certificates.X509Chain]::new()
    $chain.ChainPolicy.RevocationMode = 'NoCheck'
    [void] $chain.Build($sig.SignerCertificate)
    $root = $chain.ChainElements[$chain.ChainElements.Count - 1].Certificate.Subject
    if ($root -notmatch 'CN=Microsoft Identity Verification Root Certificate Authority 2020') { throw "$file chains to '$root', not the Microsoft Identity Verification root" }

    Write-Host "Verified: $file"
    Write-Host "  Signer:      $($sig.SignerCertificate.Subject)"
    Write-Host "  Root:        $root"
    Write-Host "  Timestamper: $($sig.TimeStamperCertificate.Subject)"
    Write-Host "  SHA-256:     $((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant())"
}
