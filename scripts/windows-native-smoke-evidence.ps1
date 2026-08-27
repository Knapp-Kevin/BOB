[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)]
  [ValidateSet('package', 'installed', 'relaunched', 'uninstalled')]
  [string]$Phase,

  [string]$InstallPath,

  [string]$InstallerPath,

  [string]$EvidencePath = (Join-Path $env:TEMP 'bob-windows-native-smoke-evidence.md')
)

$ErrorActionPreference = 'Stop'
$SessionPath = Join-Path $env:TEMP 'bob-windows-native-smoke-session.dpapi'
$SessionEntropy = [System.Text.Encoding]::UTF8.GetBytes('B.O.B. Windows native smoke session v1')

function Get-RepoHead {
  $head = (& git rev-parse HEAD 2>$null)
  if ($LASTEXITCODE -ne 0 -or -not $head) {
    throw 'Unable to resolve the current Git commit. Run this script from a B.O.B. checkout.'
  }
  return $head.Trim()
}

function Get-NormalizedPath([string]$Path) {
  if (-not $Path) { return '' }
  return [System.IO.Path]::GetFullPath($Path).TrimEnd('\')
}

function Get-RepoRoot {
  $root = (& git rev-parse --show-toplevel 2>$null)
  if ($LASTEXITCODE -ne 0 -or -not $root) {
    throw 'Unable to resolve the B.O.B. repository root. Run this script from a valid checkout.'
  }
  return Get-NormalizedPath $root.Trim()
}

function Assert-CleanRepo([string]$ExpectedHead) {
  $currentHead = Get-RepoHead
  if ($currentHead -ne $ExpectedHead) {
    throw 'The checkout HEAD changed during the native acceptance evidence session. Start again from the exact commit being accepted.'
  }

  $status = @(& git status --porcelain=v1 --untracked-files=all 2>$null)
  if ($LASTEXITCODE -ne 0) {
    throw 'Unable to inspect the Git worktree. Run this script from a valid B.O.B. checkout.'
  }
  if ($status.Count -gt 0) {
    throw 'The B.O.B. checkout is not clean. Commit, stash, or remove local changes before producing native acceptance evidence.'
  }
}

function Invoke-LockedPackageBuild([string]$ExpectedHead) {
  Assert-CleanRepo $ExpectedHead

  & npm ci
  if ($LASTEXITCODE -ne 0) {
    throw 'Locked frontend dependency installation failed. No package evidence was recorded.'
  }
  Assert-CleanRepo $ExpectedHead

  & npm run package:windows
  if ($LASTEXITCODE -ne 0) {
    throw 'The locked targeted-clean Windows package build failed. No package evidence was recorded.'
  }
  Assert-CleanRepo $ExpectedHead
}

function Get-WindowsPlatform {
  $os = Get-CimInstance Win32_OperatingSystem
  $architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
  return [pscustomobject]@{
    Version = "$($os.Caption) $($os.Version) build $($os.BuildNumber)"
    Architecture = $architecture
    IsWindows11X64 = ($os.Caption -match 'Windows 11' -and $architecture -eq 'X64')
  }
}

function Assert-AcceptancePlatform([object]$Platform) {
  if (-not $Platform.IsWindows11X64) {
    throw "Native acceptance requires Windows 11 x64. Observed '$($Platform.Version)' architecture '$($Platform.Architecture)'."
  }
}

function Get-WindowsHostIdentity {
  $machineGuid = Get-ItemPropertyValue -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Cryptography' -Name MachineGuid
  if (-not $machineGuid) {
    throw 'Unable to resolve the local Windows host identity. Native smoke evidence cannot safely continue across phases.'
  }

  $bytes = [System.Text.Encoding]::UTF8.GetBytes($machineGuid.ToString())
  $sha256 = [System.Security.Cryptography.SHA256]::Create()
  try {
    return ([System.BitConverter]::ToString($sha256.ComputeHash($bytes))).Replace('-', '').ToLowerInvariant()
  }
  finally {
    $sha256.Dispose()
  }
}

function Get-Installer {
  $installers = @(Get-ChildItem '.\src-tauri\target\release\bundle\nsis\*.exe' -File -ErrorAction SilentlyContinue)
  if ($installers.Count -ne 1) {
    throw "Expected exactly one NSIS installer, found $($installers.Count). The package phase must produce one unambiguous installer."
  }
  return $installers[0]
}

function Get-PackagedExecutable {
  $executablePath = '.\src-tauri\target\release\bob.exe'
  if (-not (Test-Path -LiteralPath $executablePath -PathType Leaf)) {
    throw 'The package build did not leave the expected src-tauri\target\release\bob.exe payload available for provenance binding.'
  }
  return Get-Item -LiteralPath $executablePath
}

function Get-TauriConfig {
  $configPath = '.\src-tauri\tauri.conf.json'
  if (-not (Test-Path -LiteralPath $configPath -PathType Leaf)) {
    throw 'Unable to find src-tauri\tauri.conf.json. Run this script from the B.O.B. repository root.'
  }

  $windowsConfigPath = '.\src-tauri\tauri.windows.conf.json'
  if (Test-Path -LiteralPath $windowsConfigPath -PathType Leaf) {
    throw 'A platform-specific Tauri Windows config is present. Reconcile the effective NSIS install target before using this acceptance helper.'
  }

  return Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json
}

function Get-CanonicalStatePath {
  if (-not $env:APPDATA) {
    throw 'APPDATA is unavailable; cannot resolve Tauri app_data_dir on Windows.'
  }

  $config = Get-TauriConfig
  $identifier = $config.identifier
  if (-not $identifier) {
    throw 'Tauri bundle identifier is missing from src-tauri\tauri.conf.json.'
  }

  return Join-Path (Join-Path $env:APPDATA $identifier) 'bob.sqlite3'
}

function Test-SamePath([string]$Left, [string]$Right) {
  if (-not $Left -or -not $Right) { return $false }
  return (Get-NormalizedPath $Left).Equals((Get-NormalizedPath $Right), [System.StringComparison]::OrdinalIgnoreCase)
}

function Test-PathAtOrBelow([string]$CandidatePath, [string]$RootPath) {
  if (-not $CandidatePath -or -not $RootPath) { return $false }
  $candidate = Get-NormalizedPath $CandidatePath
  $root = Get-NormalizedPath $RootPath
  if ($candidate.Equals($root, [System.StringComparison]::OrdinalIgnoreCase)) {
    return $true
  }
  $prefix = $root + [System.IO.Path]::DirectorySeparatorChar
  return $candidate.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)
}

function Assert-NoReparsePointPath([string]$CandidatePath, [string]$Label) {
  if (-not $CandidatePath) {
    throw "$Label must resolve to a concrete path."
  }

  $currentPath = [System.IO.Path]::GetFullPath($CandidatePath)
  while ($currentPath) {
    if (Test-Path -LiteralPath $currentPath) {
      $item = Get-Item -LiteralPath $currentPath -Force
      if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "$Label must not traverse a Windows reparse point. Use an ordinary non-junction, non-symbolic-link path for native acceptance evidence."
      }
    }

    $parent = [System.IO.Directory]::GetParent($currentPath)
    if ($null -eq $parent) { break }
    $parentPath = $parent.FullName
    if ($parentPath.Equals($currentPath, [System.StringComparison]::OrdinalIgnoreCase)) { break }
    $currentPath = $parentPath
  }
}

function Assert-NoReparsePointBelowTrustedRoot([string]$CandidatePath, [string]$TrustedRoot, [string]$Label) {
  if (-not $CandidatePath -or -not $TrustedRoot) {
    throw "$Label and its trusted root must resolve to concrete paths."
  }

  $candidate = [System.IO.Path]::GetFullPath($CandidatePath)
  $root = [System.IO.Path]::GetFullPath($TrustedRoot)
  if (-not (Test-PathAtOrBelow $candidate $root)) {
    throw "$Label must remain beneath its expected Windows profile root."
  }

  $currentPath = $candidate
  while (-not (Test-SamePath $currentPath $root)) {
    if (Test-Path -LiteralPath $currentPath) {
      $item = Get-Item -LiteralPath $currentPath -Force
      if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "$Label must not traverse a Windows reparse point below the trusted profile root. Use a clean disposable Windows profile with an ordinary B.O.B. application-data target."
      }
    }

    $parent = [System.IO.Directory]::GetParent($currentPath)
    if ($null -eq $parent) {
      throw "$Label could not be walked back to its expected Windows profile root."
    }
    $currentPath = $parent.FullName
  }
}

function Assert-EvidenceSink([string]$RepoRoot, [string]$BoundEvidencePath, [string]$BoundSessionPath) {
  if (-not $BoundEvidencePath) {
    throw 'EvidencePath must resolve to a concrete file path.'
  }
  if (Test-SamePath $BoundEvidencePath $BoundSessionPath) {
    throw 'EvidencePath must not equal the local protected smoke-session path. Keep public evidence and private continuity state separate.'
  }

  Assert-NoReparsePointPath $RepoRoot 'The B.O.B. repository checkout path'
  Assert-NoReparsePointPath $BoundEvidencePath 'EvidencePath'
  Assert-NoReparsePointPath $BoundSessionPath 'The protected smoke-session path'

  if (Test-PathAtOrBelow $BoundEvidencePath $RepoRoot) {
    throw 'EvidencePath must be outside the B.O.B. repository checkout so evidence collection cannot dirty or overwrite repository state.'
  }
  if (Test-PathAtOrBelow $BoundSessionPath $RepoRoot) {
    throw 'The protected smoke-session path must be outside the B.O.B. repository checkout so private continuity state cannot dirty or enter repository state.'
  }
}

function Assert-AcceptanceTargetSinks([string]$BoundEvidencePath, [string]$BoundSessionPath, [string]$CanonicalStatePath, [string]$DefaultInstallPath) {
  $canonicalAppDataRoot = Get-NormalizedPath (Split-Path -Parent $CanonicalStatePath)

  foreach ($sink in @(
    @{ Path = $BoundEvidencePath; Label = 'EvidencePath' },
    @{ Path = $BoundSessionPath; Label = 'The protected smoke-session path' }
  )) {
    if (Test-PathAtOrBelow $sink.Path $canonicalAppDataRoot) {
      throw "$($sink.Label) must be outside B.O.B.'s canonical application-data directory so evidence tooling cannot contaminate clean first-run state or managed recovery data."
    }
    if (Test-PathAtOrBelow $sink.Path $DefaultInstallPath) {
      throw "$($sink.Label) must be outside the package-derived default installation target so evidence tooling cannot contaminate the clean install premise."
    }
  }
}

function Get-DefaultNsisInstallPath {
  if (-not $env:LOCALAPPDATA) {
    throw 'LOCALAPPDATA is unavailable; cannot resolve the default current-user NSIS installation target.'
  }

  $config = Get-TauriConfig
  if (-not $config.productName) {
    throw 'Tauri productName is missing from src-tauri\tauri.conf.json.'
  }

  $nsis = $null
  if ($config.bundle -and $config.bundle.windows) {
    $nsis = $config.bundle.windows.nsis
  }

  if ($nsis -and $nsis.template) {
    throw 'A custom NSIS template is configured. The stock current-user default install target cannot be derived safely; reconcile issue #84 before continuing.'
  }
  if ($nsis -and $nsis.installerHooks) {
    throw 'Custom NSIS installer hooks are configured. The default install-path contract may be altered; reconcile issue #84 before continuing.'
  }

  $installMode = 'currentUser'
  if ($nsis -and $nsis.installMode) {
    $installMode = $nsis.installMode.ToString()
  }
  if ($installMode -ne 'currentUser') {
    throw "Native first-alpha acceptance requires Tauri NSIS currentUser mode. Observed installMode '$installMode'."
  }

  $normalizedProductName = ($config.productName.ToString() -replace '[\. ]+$', '')
  if (-not $normalizedProductName) {
    throw 'Tauri productName becomes empty after Windows trailing-period/space normalization.'
  }

  return Get-NormalizedPath (Join-Path $env:LOCALAPPDATA $normalizedProductName)
}

function ConvertTo-EvidencePath([string]$Path) {
  if (-not $Path) { return '' }

  $roots = @(
    @{ Value = $env:LOCALAPPDATA; Label = '%LOCALAPPDATA%' },
    @{ Value = $env:APPDATA; Label = '%APPDATA%' },
    @{ Value = $env:USERPROFILE; Label = '%USERPROFILE%' }
  ) | Where-Object { $_.Value } | Sort-Object { $_.Value.Length } -Descending

  foreach ($root in $roots) {
    $normalizedRoot = $root.Value.TrimEnd('\')
    if ($Path.Equals($normalizedRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
      return $root.Label
    }

    $prefix = $normalizedRoot + '\'
    if ($Path.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)) {
      return $root.Label + $Path.Substring($normalizedRoot.Length)
    }
  }

  return '<outside-user-profile path redacted>'
}

function Escape-Cell([object]$Value) {
  if ($null -eq $Value) { return '' }
  $escaped = $Value.ToString().Replace('|', '\|')
  return $escaped -replace "`r?`n", ' '
}

function Add-EvidenceRow([string]$Name, [object]$Value) {
  $line = "| $(Escape-Cell $Name) | $(Escape-Cell $Value) |"
  Add-Content -LiteralPath $EvidencePath -Value $line -Encoding UTF8
}

function Add-PhaseHeader([string]$CurrentPhase, [string]$CommitSha, [string]$WindowsVersion, [string]$WindowsArchitecture, [string]$Timestamp) {
  Add-EvidenceRow 'Evidence snapshot UTC' $Timestamp
  Add-EvidenceRow 'Phase' $CurrentPhase
  Add-EvidenceRow 'Exact commit SHA' $CommitSha
  Add-EvidenceRow 'Windows version/build' $WindowsVersion
  Add-EvidenceRow 'Windows architecture' $WindowsArchitecture
}

function Initialize-EvidenceFile {
  @(
    '# B.O.B. Windows native smoke evidence',
    '',
    '> Generated by `scripts/windows-native-smoke-evidence.ps1`. Machine-observed fields are evidence helpers, not substitutes for the manual acceptance steps in `docs/WINDOWS_NSIS_SMOKE.md`. Local filesystem paths are redacted or represented with environment-variable roots before being written.',
    '',
    '| Evidence | Result |',
    '| --- | --- |'
  ) | Set-Content -LiteralPath $EvidencePath -Encoding UTF8
}

function Get-EvidenceDigest {
  if (-not (Test-Path -LiteralPath $EvidencePath -PathType Leaf)) {
    throw 'The bound evidence file is missing. Start a new package phase instead of continuing a broken evidence chain.'
  }
  return (Get-FileHash -LiteralPath $EvidencePath -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Protect-SmokeSessionJson([string]$Json) {
  $plainBytes = [System.Text.Encoding]::UTF8.GetBytes($Json)
  try {
    $protectedBytes = [System.Security.Cryptography.ProtectedData]::Protect(
      $plainBytes,
      $SessionEntropy,
      [System.Security.Cryptography.DataProtectionScope]::CurrentUser
    )
    [System.IO.File]::WriteAllBytes($SessionPath, $protectedBytes)
  }
  catch {
    throw 'Unable to protect the Windows smoke session with user-scoped DPAPI. No plaintext session fallback is allowed.'
  }
  finally {
    if ($plainBytes) {
      [System.Array]::Clear($plainBytes, 0, $plainBytes.Length)
    }
  }
}

function Unprotect-SmokeSessionJson {
  if (-not (Test-Path -LiteralPath $SessionPath -PathType Leaf)) {
    throw 'No active protected Windows smoke session exists. Run the package phase first for the exact package under acceptance.'
  }

  try {
    $protectedBytes = [System.IO.File]::ReadAllBytes($SessionPath)
    if (-not $protectedBytes -or $protectedBytes.Length -eq 0) {
      throw 'The protected smoke-session file is empty.'
    }
    $plainBytes = [System.Security.Cryptography.ProtectedData]::Unprotect(
      $protectedBytes,
      $SessionEntropy,
      [System.Security.Cryptography.DataProtectionScope]::CurrentUser
    )
    try {
      return [System.Text.Encoding]::UTF8.GetString($plainBytes)
    }
    finally {
      if ($plainBytes) {
        [System.Array]::Clear($plainBytes, 0, $plainBytes.Length)
      }
    }
  }
  catch {
    throw 'Unable to unprotect the Windows smoke session. It may be tampered, corrupt, or belong to a different Windows user. Start a new package phase instead of continuing this acceptance chain.'
  }
}

function Save-SmokeSession([string]$CommitSha, [string]$WindowsVersion, [string]$WindowsArchitecture, [string]$HostIdentity, [string]$CanonicalStatePath, [string]$InstallerFilename, [string]$InstallerHash, [string]$PackagedExecutableHash, [string]$BoundInstallPath, [string]$BoundEvidencePath, [string]$EvidenceDigest) {
  $session = [ordered]@{
    commitSha = $CommitSha
    windowsVersion = $WindowsVersion
    windowsArchitecture = $WindowsArchitecture
    hostIdentity = $HostIdentity
    canonicalStatePath = (Get-NormalizedPath $CanonicalStatePath)
    evidencePath = (Get-NormalizedPath $BoundEvidencePath)
    evidenceDigest = $EvidenceDigest
    installerFilename = $InstallerFilename
    installerHash = $InstallerHash
    packagedExecutableHash = $PackagedExecutableHash
    installPath = (Get-NormalizedPath $BoundInstallPath)
    installedProcessId = $null
    installedProcessStartTimeUtc = $null
    phase = 'packaged'
    startedUtc = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ss.fffffffZ')
  }
  Protect-SmokeSessionJson ($session | ConvertTo-Json -Compress)
}

function Get-SmokeSession {
  try {
    return (Unprotect-SmokeSessionJson) | ConvertFrom-Json
  }
  catch {
    if ($_.Exception.Message -like 'Unable to unprotect the Windows smoke session*' -or $_.Exception.Message -like 'No active protected Windows smoke session exists*') {
      throw
    }
    throw 'The protected Windows smoke session could not be parsed. Start a new package phase instead of continuing malformed continuity state.'
  }
}

function Save-SmokeSessionObject([object]$Session) {
  Protect-SmokeSessionJson ($Session | ConvertTo-Json -Compress)
}

function Assert-EvidenceDigest([object]$Session) {
  if (-not $Session.evidenceDigest) {
    throw 'The smoke session does not contain an evidence-file digest. Start a new package phase with the current helper.'
  }

  $currentDigest = Get-EvidenceDigest
  if ($currentDigest -ne $Session.evidenceDigest) {
    throw 'The bound evidence file changed outside the active smoke phase. Start a new package phase instead of continuing modified, replaced, or truncated acceptance evidence.'
  }
}

function Update-SessionEvidenceDigest([object]$Session) {
  $Session | Add-Member -NotePropertyName evidenceDigest -NotePropertyValue (Get-EvidenceDigest) -Force
  Save-SmokeSessionObject $Session
}

function Assert-SmokeSession([object]$Session, [string]$CommitSha, [string]$WindowsVersion, [string]$WindowsArchitecture, [string]$HostIdentity, [string]$CanonicalStatePath, [string]$DefaultInstallPath, [string]$BoundEvidencePath) {
  if ($Session.commitSha -ne $CommitSha) {
    throw 'The current checkout SHA does not match the package-phase smoke session. Start a new package phase instead of mixing acceptance heads.'
  }
  if ($Session.windowsVersion -ne $WindowsVersion) {
    throw 'The current Windows version/build does not match the package-phase smoke session. Start a new package phase instead of mixing acceptance environments.'
  }
  if ($Session.windowsArchitecture -ne $WindowsArchitecture) {
    throw 'The current Windows architecture does not match the package-phase smoke session. Start a new package phase instead of mixing acceptance platforms.'
  }
  if ($Session.hostIdentity -ne $HostIdentity) {
    throw 'The current Windows host does not match the package-phase smoke session. Start a new package phase on this host instead of mixing acceptance machines.'
  }
  if (-not (Test-SamePath $Session.canonicalStatePath $CanonicalStatePath)) {
    throw 'The current canonical-state target does not match the package-phase smoke session. Start a new package phase instead of mixing Windows profiles or package identifiers.'
  }
  if (-not (Test-SamePath $Session.installPath $DefaultInstallPath)) {
    throw 'The current package-derived default install target does not match the package-phase smoke session. Start a new package phase instead of changing package path policy mid-session.'
  }
  if (-not (Test-SamePath $Session.evidencePath $BoundEvidencePath)) {
    throw 'The current evidence path does not match the package-phase smoke session. Use the same evidence file for every phase or start a new package phase.'
  }
}

function Assert-SessionPhase([object]$Session, [string]$ExpectedPhase, [string]$RequestedPhase) {
  if ($Session.phase -ne $ExpectedPhase) {
    throw "The '$RequestedPhase' evidence phase is out of order. Expected active smoke-session phase '$ExpectedPhase', observed '$($Session.phase)'. Start a new package phase if the acceptance sequence was interrupted."
  }
}

function Begin-SessionPhase([object]$Session, [string]$ExpectedPhase, [string]$RequestedPhase) {
  Assert-SessionPhase $Session $ExpectedPhase $RequestedPhase
  Assert-EvidenceDigest $Session
  $Session | Add-Member -NotePropertyName phase -NotePropertyValue "$RequestedPhase-writing" -Force
  Save-SmokeSessionObject $Session
}

function Complete-SessionPhase([object]$Session, [string]$NextPhase) {
  $Session | Add-Member -NotePropertyName phase -NotePropertyValue $NextPhase -Force
  Save-SmokeSessionObject $Session
  Update-SessionEvidenceDigest $Session
}

function Assert-InstallerMatchesSession([object]$Session, [string]$CandidateInstallerPath) {
  if (-not $CandidateInstallerPath) {
    throw '-InstallerPath is required for the installed phase so the supplied installer file can be checked against the package-phase hash.'
  }
  if (-not (Test-Path -LiteralPath $CandidateInstallerPath -PathType Leaf)) {
    throw 'The supplied installer path does not exist or is not a file.'
  }

  $candidateHash = (Get-FileHash -LiteralPath $CandidateInstallerPath -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($candidateHash -ne $Session.installerHash) {
    throw 'The supplied installer does not match the package-phase SHA-256. Start a new package phase or install the exact bound package instead of mixing installers.'
  }
  return $candidateHash
}

function Assert-InstalledExecutableMatchesSession([object]$Session, [string]$BoundInstallPath) {
  if (-not $Session.packagedExecutableHash) {
    throw 'The smoke session does not contain packaged executable identity. Start a new package phase with the current helper.'
  }
  $installedExecutable = Join-Path $BoundInstallPath 'bob.exe'
  Assert-NoReparsePointBelowTrustedRoot $installedExecutable $BoundInstallPath 'The installed B.O.B. executable'
  if (-not (Test-Path -LiteralPath $installedExecutable -PathType Leaf)) {
    throw 'The expected installed bob.exe does not exist at the recorded install path.'
  }

  $installedHash = (Get-FileHash -LiteralPath $installedExecutable -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($installedHash -ne $Session.packagedExecutableHash) {
    throw 'The installed bob.exe does not match the package-built executable SHA-256. Leave issue #84 open and investigate package/install provenance instead of advancing the smoke session.'
  }
  return $installedHash
}

function Get-InstalledBobProcesses([string]$BoundInstallPath) {
  if (-not $BoundInstallPath) { return @() }
  $expectedExecutable = Get-NormalizedPath (Join-Path $BoundInstallPath 'bob.exe')

  try {
    $processes = @(Get-Process bob -ErrorAction Stop)
  }
  catch {
    if ($_.FullyQualifiedErrorId -like 'NoProcessFoundForGivenName*') {
      return @()
    }
    throw 'Unable to enumerate bob.exe processes. Process absence is not proven; leave issue #84 open and investigate process inspection before continuing.'
  }

  $matching = @()
  foreach ($process in $processes) {
    try {
      $processPath = $process.Path
    }
    catch {
      throw 'Unable to inspect a bob.exe process executable path. Bound-process state is unknown; leave issue #84 open and investigate process inspection before continuing.'
    }

    if (-not $processPath) {
      throw 'A bob.exe process did not expose an executable path. Bound-process state is unknown; leave issue #84 open and investigate process inspection before continuing.'
    }

    if (Test-SamePath $processPath $expectedExecutable) {
      $matching += $process
    }
  }

  return $matching
}

function Get-ProcessStartTimeUtc([object]$Process) {
  return $Process.StartTime.ToUniversalTime().ToString('o')
}

$head = Get-RepoHead
Assert-CleanRepo $head
$repoRoot = Get-RepoRoot
$normalizedEvidencePath = Get-NormalizedPath $EvidencePath
$normalizedSessionPath = Get-NormalizedPath $SessionPath
Assert-EvidenceSink $repoRoot $normalizedEvidencePath $normalizedSessionPath

$windowsPlatform = Get-WindowsPlatform
$windowsVersion = $windowsPlatform.Version
$windowsArchitecture = $windowsPlatform.Architecture
$hostIdentity = Get-WindowsHostIdentity
$timestamp = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
$canonicalStatePath = Get-CanonicalStatePath
Assert-NoReparsePointBelowTrustedRoot $canonicalStatePath $env:APPDATA 'The canonical B.O.B. application-data target'
$defaultInstallPath = Get-DefaultNsisInstallPath
Assert-NoReparsePointBelowTrustedRoot $defaultInstallPath $env:LOCALAPPDATA 'The default B.O.B. installation target'
Assert-AcceptanceTargetSinks $normalizedEvidencePath $normalizedSessionPath $canonicalStatePath $defaultInstallPath

if ($Phase -eq 'package') {
  Assert-AcceptancePlatform $windowsPlatform

  if (Test-Path -LiteralPath $SessionPath -PathType Leaf) {
    Remove-Item -LiteralPath $SessionPath -Force
  }

  if ($InstallPath -and -not (Test-SamePath $InstallPath $defaultInstallPath)) {
    throw '-InstallPath, when supplied, must match the package-derived default current-user NSIS target. Custom install targets do not satisfy issue #84.'
  }
  if (Test-Path -LiteralPath $defaultInstallPath) {
    throw 'The package-derived default B.O.B. installation target already exists. Use a disposable clean test profile; do not delete an existing installation merely to satisfy this smoke test.'
  }

  try {
    $canonicalStateObject = Get-Item -LiteralPath $canonicalStatePath -Force -ErrorAction Stop
  }
  catch [System.Management.Automation.ItemNotFoundException] {
    $canonicalStateObject = $null
  }
  catch {
    throw 'Unable to prove the canonical B.O.B. state target is absent. Use a clean disposable Windows profile and investigate the filesystem state before continuing.'
  }
  if ($null -ne $canonicalStateObject) {
    throw 'The canonical B.O.B. state target already contains a filesystem object. Use a disposable clean test profile; do not delete real user data or other existing state merely to satisfy this smoke test.'
  }

  Invoke-LockedPackageBuild $head
  $installer = Get-Installer
  $packagedExecutable = Get-PackagedExecutable
  $hash = Get-FileHash -LiteralPath $installer.FullName -Algorithm SHA256
  $packagedExecutableHash = (Get-FileHash -LiteralPath $packagedExecutable.FullName -Algorithm SHA256).Hash.ToLowerInvariant()

  Initialize-EvidenceFile
  Add-PhaseHeader $Phase $head $windowsVersion $windowsArchitecture $timestamp
  Add-EvidenceRow 'Acceptance platform verified' 'Windows 11 x64'
  Add-EvidenceRow 'NSIS install mode verified' 'currentUser'
  Add-EvidenceRow 'Package dependency/build path' 'npm ci + npm run package:windows (locked targeted-clean)'
  Add-EvidenceRow 'Expected canonical state path' (ConvertTo-EvidencePath $canonicalStatePath)
  Add-EvidenceRow 'Expected canonical state absent before install' $true
  Add-EvidenceRow 'Package-derived default install path' (ConvertTo-EvidencePath $defaultInstallPath)
  Add-EvidenceRow 'Default install target absent before install' $true
  Add-EvidenceRow 'Installer filename' $installer.Name
  Add-EvidenceRow 'Installer SHA-256' $hash.Hash.ToLowerInvariant()
  Add-EvidenceRow 'Package-built bob.exe SHA-256' $packagedExecutableHash

  $evidenceDigest = Get-EvidenceDigest
  Save-SmokeSession $head $windowsVersion $windowsArchitecture $hostIdentity $canonicalStatePath $installer.Name $hash.Hash.ToLowerInvariant() $packagedExecutableHash $defaultInstallPath $normalizedEvidencePath $evidenceDigest
}
else {
  $session = Get-SmokeSession
  Assert-SmokeSession $session $head $windowsVersion $windowsArchitecture $hostIdentity $canonicalStatePath $defaultInstallPath $normalizedEvidencePath

  switch ($Phase) {
    'installed' {
      Assert-SessionPhase $session 'packaged' $Phase
      if ($InstallPath -and -not (Test-SamePath $InstallPath $defaultInstallPath)) {
        throw '-InstallPath, when supplied, does not match the package-derived default current-user NSIS target.'
      }
      $verifiedInstallerHash = Assert-InstallerMatchesSession $session $InstallerPath
      $normalizedInstallPath = $defaultInstallPath
      if (-not (Test-Path -LiteralPath $normalizedInstallPath -PathType Container)) { throw 'The package-derived default install path does not exist or is not a directory.' }

      if (-not $session.installPath -or -not (Test-SamePath $session.installPath $normalizedInstallPath)) {
        throw 'The observed default install path does not match the clean package-derived target bound during the package phase.'
      }

      $verifiedInstalledExecutableHash = Assert-InstalledExecutableMatchesSession $session $normalizedInstallPath
      try {
        $installDb = @(Get-ChildItem -LiteralPath $normalizedInstallPath -Filter 'bob.sqlite3' -File -Recurse -ErrorAction Stop)
      }
      catch {
        throw 'Unable to fully enumerate the installation directory while checking for bob.sqlite3. Database absence is not proven; leave issue #84 open and investigate the unreadable path before continuing.'
      }
      $bobProcesses = @(Get-InstalledBobProcesses $normalizedInstallPath)
      $canonicalStateExists = Test-Path -LiteralPath $canonicalStatePath -PathType Leaf

      if ($bobProcesses.Count -ne 1) {
        throw "Expected exactly one running bob.exe from the default install directory while capturing the installed phase; observed $($bobProcesses.Count). Resolve duplicate or missing processes before continuing."
      }
      if (-not $canonicalStateExists) {
        throw 'The expected canonical bob.sqlite3 was not created. Leave issue #84 open and investigate instead of advancing the smoke session.'
      }
      if ($installDb.Count -ne 0) {
        throw 'bob.sqlite3 was found under the installation directory. Canonical state must remain in the application-data location.'
      }

      $installedProcess = $bobProcesses[0]
      $installedProcessStartTimeUtc = Get-ProcessStartTimeUtc $installedProcess

      Begin-SessionPhase $session 'packaged' $Phase
      Add-PhaseHeader $Phase $head $windowsVersion $windowsArchitecture $timestamp
      Add-EvidenceRow 'Supplied installer matches package-phase SHA-256' ($verifiedInstallerHash -eq $session.installerHash)
      Add-EvidenceRow 'Installed bob.exe matches package-built SHA-256' ($verifiedInstalledExecutableHash -eq $session.packagedExecutableHash)
      Add-EvidenceRow 'Install path matches package-derived default target' $true
      Add-EvidenceRow 'Install path' (ConvertTo-EvidencePath $normalizedInstallPath)
      Add-EvidenceRow 'Installed B.O.B. process running from default install path' $true
      Add-EvidenceRow 'Expected canonical state path' (ConvertTo-EvidencePath $canonicalStatePath)
      Add-EvidenceRow 'Expected canonical state exists' $true
      Add-EvidenceRow 'Database absent from install directory' $true

      $session | Add-Member -NotePropertyName installedProcessId -NotePropertyValue $installedProcess.Id -Force
      $session | Add-Member -NotePropertyName installedProcessStartTimeUtc -NotePropertyValue $installedProcessStartTimeUtc -Force
      Complete-SessionPhase $session 'installed'
    }

    'relaunched' {
      Assert-SessionPhase $session 'installed' $Phase
      if (-not $session.installPath) {
        throw 'No install path is bound to this smoke session. Run the package phase again with the current helper.'
      }
      if ($null -eq $session.installedProcessId -or -not $session.installedProcessStartTimeUtc) {
        throw 'Installed process identity is missing from this smoke session. Start a new package phase with the current helper before recording relaunch evidence.'
      }

      $verifiedRelaunchedExecutableHash = Assert-InstalledExecutableMatchesSession $session $session.installPath
      $installedStart = [DateTimeOffset]::Parse($session.installedProcessStartTimeUtc)
      $bobProcesses = @(Get-InstalledBobProcesses $session.installPath)
      $canonicalStateExists = Test-Path -LiteralPath $canonicalStatePath -PathType Leaf

      if ($bobProcesses.Count -ne 1) {
        throw "Expected exactly one running bob.exe from the recorded install directory after relaunch; observed $($bobProcesses.Count). Fully quit the original process and resolve duplicate or missing processes before continuing."
      }

      $relaunchedProcess = $bobProcesses[0]
      $relaunchedStart = [DateTimeOffset]$relaunchedProcess.StartTime.ToUniversalTime()
      if ($relaunchedProcess.Id -eq [int]$session.installedProcessId -or $relaunchedStart -le $installedStart) {
        throw 'The running bob.exe is not a fresh process after the installed snapshot. Fully quit the original process, relaunch B.O.B., and capture this phase while the new process is running.'
      }
      if (-not $canonicalStateExists) {
        throw 'The expected canonical bob.sqlite3 is missing after relaunch. Leave issue #84 open and investigate instead of advancing the smoke session.'
      }

      Begin-SessionPhase $session 'installed' $Phase
      Add-PhaseHeader $Phase $head $windowsVersion $windowsArchitecture $timestamp
      Add-EvidenceRow 'Relaunched bob.exe matches package-built SHA-256' ($verifiedRelaunchedExecutableHash -eq $session.packagedExecutableHash)
      Add-EvidenceRow 'Fresh B.O.B. process observed after relaunch' $true
      Add-EvidenceRow 'Expected canonical state path' (ConvertTo-EvidencePath $canonicalStatePath)
      Add-EvidenceRow 'Expected canonical state exists after relaunch' $true

      Complete-SessionPhase $session 'relaunched'
    }

    'uninstalled' {
      Assert-SessionPhase $session 'relaunched' $Phase
      if ($InstallPath -and -not (Test-SamePath $InstallPath $defaultInstallPath)) {
        throw '-InstallPath, when supplied, does not match the package-derived default current-user NSIS target.'
      }
      if (-not $session.installPath) {
        throw 'No install path is bound to this smoke session. Start a new package phase with the current helper.'
      }

      $normalizedInstallPath = $defaultInstallPath
      if (-not (Test-SamePath $session.installPath $normalizedInstallPath)) {
        throw 'The uninstall-check path does not match the package-derived default target bound to this smoke session.'
      }

      $bobProcesses = @(Get-InstalledBobProcesses $session.installPath)
      $installPathRemoved = -not (Test-Path -LiteralPath $normalizedInstallPath)
      $canonicalStateRetained = Test-Path -LiteralPath $canonicalStatePath -PathType Leaf

      if ($bobProcesses.Count -ne 0) {
        throw "Expected no running bob.exe from the recorded install directory after uninstall; observed $($bobProcesses.Count). Fully stop B.O.B. and leave issue #84 open instead of advancing the smoke session."
      }
      if (-not $installPathRemoved) {
        throw 'The recorded installation directory still exists after uninstall. Leave issue #84 open and investigate instead of advancing the smoke session.'
      }
      if (-not $canonicalStateRetained) {
        throw 'The expected canonical bob.sqlite3 was not retained after uninstall. Leave issue #84 open and investigate instead of advancing the smoke session.'
      }

      Begin-SessionPhase $session 'relaunched' $Phase
      Add-PhaseHeader $Phase $head $windowsVersion $windowsArchitecture $timestamp
      Add-EvidenceRow 'No B.O.B. process remains after uninstall' $true
      Add-EvidenceRow 'Default install path removed' $true
      Add-EvidenceRow 'Expected canonical state path' (ConvertTo-EvidencePath $canonicalStatePath)
      Add-EvidenceRow 'Expected canonical state retained' $true

      Complete-SessionPhase $session 'uninstalled'
    }
  }
}

Write-Host 'Evidence appended.'
Write-Host 'Smoke session state is stored locally under %TEMP% using Windows user-scoped DPAPI and is not public evidence.'
Write-Host 'Record the remaining manual observations required by docs/WINDOWS_NSIS_SMOKE.md before closing issue #84.'