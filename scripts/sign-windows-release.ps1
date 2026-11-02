# Signs one file with the MythologIQ Labs Azure Artifact Signing profile. Tauri runs it
# through bundle.windows.signCommand in src-tauri/tauri.release-windows.conf.json for
# bob.exe, the NSIS installer and its uninstaller, and only for release builds.
#
# It needs BOB_SIGNING_CLIENT_DIR (the extracted Microsoft.ArtifactSigning.Client NuGet
# package) and an Azure CLI sign-in with the Signer role: azure/login with GitHub OIDC
# in the release workflow, or az login locally. It fails closed when either is missing.
# BOB_SIGNTOOL may name signtool.exe; otherwise the newest Windows SDK x64 copy is used.
# Procedure: MythologIQ-Labs-LLC/Myth-Tech-Forge docs/WINDOWS_CODE_SIGNING_STANDARD.md.
param(
    [Parameter(Mandatory)] [string] $Path
)
$ErrorActionPreference = 'Stop'

if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "nothing to sign at $Path" }

$clientDir = $env:BOB_SIGNING_CLIENT_DIR
if (-not $clientDir) { throw 'BOB_SIGNING_CLIENT_DIR is not set; refusing to build an unsigned release' }
$dlib = Join-Path $clientDir 'bin\x64\Azure.CodeSigning.Dlib.dll'
if (-not (Test-Path -LiteralPath $dlib)) { throw "Artifact Signing client not found at $dlib" }

$signtool = $env:BOB_SIGNTOOL
if (-not $signtool) {
    $signtool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" -ErrorAction SilentlyContinue |
        Sort-Object FullName -Descending | Select-Object -First 1 -ExpandProperty FullName
}
if (-not $signtool -or -not (Test-Path -LiteralPath $signtool)) { throw 'signtool.exe not found; install the Windows SDK or set BOB_SIGNTOOL' }

# Only the Azure CLI credential is allowed, so the signer is whoever az is signed in as.
# No byte-order mark: the signing client cannot parse one (Windows PowerShell's
# -Encoding utf8 always writes it).
$metadata = Join-Path $clientDir 'bob-signing-metadata.json'
$json = [ordered]@{
    Endpoint               = 'https://eus.codesigning.azure.net/'
    CodeSigningAccountName = 'MythologIQ-Labs-Signing'
    CertificateProfileName = 'mythologiq-labs-public-release'
    ExcludeCredentials     = @(
        'ManagedIdentityCredential', 'EnvironmentCredential', 'WorkloadIdentityCredential',
        'SharedTokenCacheCredential', 'VisualStudioCredential', 'VisualStudioCodeCredential',
        'AzurePowerShellCredential', 'AzureDeveloperCliCredential', 'InteractiveBrowserCredential'
    )
} | ConvertTo-Json
[IO.File]::WriteAllText($metadata, $json)

& $signtool sign /v /fd SHA256 /tr http://timestamp.acs.microsoft.com /td SHA256 /dlib $dlib /dmdf $metadata $Path
if ($LASTEXITCODE -ne 0) { throw "signtool sign failed for $Path (exit $LASTEXITCODE)" }
