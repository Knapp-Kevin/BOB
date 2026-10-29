# Read-only validation of an installed B.O.B. Windows executable.
# Run after the NSIS installer completes on a qualification machine.
# Does not launch B.O.B. or modify the user profile.
param(
    [Parameter(Mandatory = $true)]
    [string] $Executable,
    [string] $Installer,
    [string] $EvidencePath,
    [ValidatePattern("^[0-9a-fA-F]{64}$")]
    [string] $ExpectedExecutableSHA256,
    [ValidatePattern("^[0-9a-fA-F]{64}$")]
    [string] $ExpectedInstallerSHA256
)
$ErrorActionPreference = 'Stop'
$resolved = (Resolve-Path -LiteralPath $Executable -ErrorAction Stop).Path
$stream = [System.IO.File]::Open($resolved, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::Read)
try {
    $reader = [System.IO.BinaryReader]::new($stream)
    try {
        if ($reader.ReadUInt16() -ne 0x5A4D) { throw 'Not an MZ executable' }
        $stream.Position = 0x3C
        $peOffset = $reader.ReadInt32()
        if ($peOffset -lt 64 -or $peOffset -gt ($stream.Length - 100)) { throw 'Invalid PE header offset' }
        $stream.Position = $peOffset
        if ($reader.ReadUInt32() -ne 0x00004550) { throw 'Not a PE executable' }
        # COFF FileHeader: SizeOfOptionalHeader at +20 from the PE signature.
        $stream.Position = $peOffset + 20
        $optionalSize = $reader.ReadUInt16()
        if ($optionalSize -lt 70 -or ($peOffset + 24 + $optionalSize) -gt $stream.Length) {
            throw "Invalid or truncated PE optional header"
        }
        $stream.Position = $peOffset + 24
        $magic = $reader.ReadUInt16()
        if ($magic -notin @(0x10B, 0x20B)) { throw 'Unknown PE optional header format' }
        $stream.Position = $peOffset + 24 + 68
        $subsystem = $reader.ReadUInt16()
        $hash = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash
        $signature = Get-AuthenticodeSignature -LiteralPath $resolved
        $executableHashMatches = (-not $ExpectedExecutableSHA256) -or ($hash -ieq $ExpectedExecutableSHA256)
        $installerEvidence = $null
        if ($ExpectedInstallerSHA256 -and -not $Installer) {
            throw "An installer file is required when ExpectedInstallerSHA256 is set"
        }
        if ($Installer) {
            $installerPath = (Resolve-Path -LiteralPath $Installer -ErrorAction Stop).Path
            $installerHash = (Get-FileHash -LiteralPath $installerPath -Algorithm SHA256).Hash
            $installerEvidence = [pscustomobject]@{
                Path = $installerPath
                SHA256 = $installerHash
                ExpectedSHA256 = $(if ($ExpectedInstallerSHA256) { $ExpectedInstallerSHA256.ToUpperInvariant() } else { $null })
                HashMatchesExpected = (-not $ExpectedInstallerSHA256) -or ($installerHash -ieq $ExpectedInstallerSHA256)
                SignatureStatus = (Get-AuthenticodeSignature -LiteralPath $installerPath).Status.ToString()
            }
        }
        $result = [pscustomobject]@{
            Executable = $resolved
            SHA256 = $hash
            ExpectedSHA256 = $(if ($ExpectedExecutableSHA256) { $ExpectedExecutableSHA256.ToUpperInvariant() } else { $null })
            HashMatchesExpected = $executableHashMatches
            PESubsystem = $subsystem
            GUIExpected = 2
            SignatureStatus = $signature.Status.ToString()
            Installer = $installerEvidence
            Passed = ($subsystem -eq 2) -and $executableHashMatches -and (($null -eq $installerEvidence) -or $installerEvidence.HashMatchesExpected)
        }
        $result | Format-List
        if ($EvidencePath) {
            $evidence = [pscustomobject]@{
                CapturedAtUtc = [DateTime]::UtcNow.ToString("o")
                HostOS = [System.Environment]::OSVersion.VersionString
                Check = "Installed Windows PE GUI subsystem"
                Result = $result
            }
            $json = $evidence | ConvertTo-Json -Depth 6
            $target = [System.IO.Path]::GetFullPath($EvidencePath)
            $directory = [System.IO.Path]::GetDirectoryName($target)
            if (-not [System.IO.Directory]::Exists($directory)) {
                throw "Evidence directory does not exist: $directory"
            }
            [System.IO.File]::WriteAllText($target, $json, [System.Text.UTF8Encoding]::new($false))
            Write-Host "Evidence written to $target"
        }
        if (-not $result.Passed) { throw "Installed executable qualification failed; see PE subsystem and SHA-256 comparisons in the evidence record" }
    } finally { $reader.Dispose() }
} finally { $stream.Dispose() }
Write-Host 'PASS: installed executable uses Windows GUI subsystem (2) and all supplied SHA-256 expectations match.'
Write-Host 'This verifies PE metadata only; it does NOT establish native launch, multi-instance, or NSIS uninstall qualification.'
