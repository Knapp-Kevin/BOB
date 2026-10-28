# Read-only validation of an installed B.O.B. Windows executable.
# Run after the NSIS installer completes on a qualification machine.
# Does not launch B.O.B. or modify the user profile.
param(
    [Parameter(Mandatory = $true)]
    [string] $Executable
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
        $stream.Position = $peOffset + 24
        $magic = $reader.ReadUInt16()
        if ($magic -notin @(0x10B, 0x20B)) { throw 'Unknown PE optional header format' }
        $stream.Position = $peOffset + 24 + 68
        $subsystem = $reader.ReadUInt16()
        $hash = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash
        $result = [pscustomobject]@{
            Executable = $resolved
            SHA256 = $hash
            PESubsystem = $subsystem
            GUIExpected = 2
            Passed = ($subsystem -eq 2)
        }
        $result | Format-List
        if ($subsystem -ne 2) { throw "Installed executable is not a Windows GUI-subsystem binary: $subsystem" }
    } finally { $reader.Dispose() }
} finally { $stream.Dispose() }
Write-Host 'PASS: installed executable uses Windows GUI subsystem (2).'
Write-Host 'This is a read-only header check, NOT a launch, multi-instance, or NSIS uninstall qualification.'
