# Stages a Windows Sandbox that builds the B.O.B. NSIS installer from a committed ref,
# for hosts whose Smart App Control blocks unsigned release build tools. The Sandbox
# has networking (it downloads the toolchain) but no clipboard, printers, audio/video
# input or vGPU, and maps only a read-only source snapshot and an output folder. Run it
# non-elevated; it changes no host security setting and does not launch the Sandbox.
param(
    [string] $Ref = 'HEAD',
    [string] $KitRoot = 'C:\qual\sandbox-build'
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot
$commit = (git -C $repo rev-parse $Ref).Trim()
if ($LASTEXITCODE -ne 0) { throw "cannot resolve $Ref" }

$kitInput = Join-Path $KitRoot 'input'
$output = Join-Path $KitRoot 'output'
foreach ($dir in $kitInput, $output) {
    $full = [IO.Path]::GetFullPath($dir)
    foreach ($forbidden in $env:APPDATA, $env:LOCALAPPDATA, $env:USERPROFILE, $repo) {
        if ($full.StartsWith($forbidden, [StringComparison]::OrdinalIgnoreCase)) { throw "$full is inside $forbidden; the kit must not map it" }
    }
}
Remove-Item -Recurse -Force $kitInput, $output -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force (Join-Path $kitInput 'src'), $output | Out-Null

# A committed snapshot only: no working-tree edits, node_modules or target directory.
$archive = Join-Path $KitRoot 'src.zip'
git -C $repo archive --format=zip -o $archive $commit
if ($LASTEXITCODE -ne 0) { throw 'git archive failed' }
Expand-Archive -LiteralPath $archive -DestinationPath (Join-Path $kitInput 'src') -Force
Remove-Item -LiteralPath $archive
"$commit ($((git -C $repo log -1 --format=%s $commit).Trim()))" | Set-Content -LiteralPath (Join-Path $kitInput 'source.txt')
Copy-Item (Join-Path $PSScriptRoot 'build-in-windows-sandbox.ps1') $kitInput

# Windows Installer stalls ~120 s per package inside Windows Sandbox, so reuse the host's
# installed MSVC and Windows SDK, mapped read-only, when they exist.
$toolchainMaps = ''
$msvc = Get-ChildItem "${env:ProgramFiles(x86)}\Microsoft Visual Studio\*\*\VC\Tools\MSVC\*", "$env:ProgramFiles\Microsoft Visual Studio\*\*\VC\Tools\MSVC\*" -Directory -ErrorAction SilentlyContinue |
    Where-Object { Test-Path (Join-Path $_.FullName 'bin\Hostx64\x64\link.exe') } | Sort-Object Name | Select-Object -Last 1
$kits = "${env:ProgramFiles(x86)}\Windows Kits\10"
if ($msvc -and (Test-Path "$kits\Lib")) {
    "MSVC $($msvc.Name) from $($msvc.FullName); Windows SDK from $kits" | Set-Content -LiteralPath (Join-Path $kitInput 'toolchain.txt')
    $toolchainMaps = @"
    <MappedFolder><HostFolder>$($msvc.FullName)</HostFolder><SandboxFolder>C:\toolchain\msvc</SandboxFolder><ReadOnly>true</ReadOnly></MappedFolder>
    <MappedFolder><HostFolder>$kits</HostFolder><SandboxFolder>C:\toolchain\winsdk</SandboxFolder><ReadOnly>true</ReadOnly></MappedFolder>
"@
}

$wsb = Join-Path $KitRoot 'BOB-Installer-Build.wsb'
@"
<Configuration>
  <Networking>Default</Networking>
  <VGpu>Disable</VGpu>
  <ClipboardRedirection>Disable</ClipboardRedirection>
  <PrinterRedirection>Disable</PrinterRedirection>
  <AudioInput>Disable</AudioInput>
  <VideoInput>Disable</VideoInput>
  <ProtectedClient>Enable</ProtectedClient>
  <MemoryInMB>8192</MemoryInMB>
  <MappedFolders>
    <MappedFolder><HostFolder>$kitInput</HostFolder><SandboxFolder>C:\kit</SandboxFolder><ReadOnly>true</ReadOnly></MappedFolder>
    <MappedFolder><HostFolder>$output</HostFolder><SandboxFolder>C:\out</SandboxFolder><ReadOnly>false</ReadOnly></MappedFolder>
$toolchainMaps  </MappedFolders>
  <LogonCommand><Command>powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\kit\build-in-windows-sandbox.ps1</Command></LogonCommand>
</Configuration>
"@ | Set-Content -LiteralPath $wsb -Encoding utf8

[xml] $config = Get-Content -LiteralPath $wsb -Raw
$maps = @($config.Configuration.MappedFolders.MappedFolder)
if ($config.Configuration.ClipboardRedirection -ne 'Disable' -or $maps.Count -notin 2, 4) { throw "generated $wsb does not have the required isolation" }
foreach ($m in $maps) {
    if (-not (Test-Path -LiteralPath $m.HostFolder)) { throw "mapped folder $($m.HostFolder) is missing" }
    if ($m.SandboxFolder -ne 'C:\out' -and $m.ReadOnly -ne 'true') { throw "$($m.HostFolder) must be mapped read-only" }
}
Write-Host "Build kit ready: $wsb"
Write-Host "source $commit; mapped: $(($maps | ForEach-Object { "$($_.HostFolder) -> $($_.SandboxFolder)$(if ($_.ReadOnly -eq 'true') { ' (read-only)' })" }) -join '; ')"
